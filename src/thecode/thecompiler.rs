use crate::prelude::*;

// Some code taken from https://github.com/ceronman/loxido/blob/master/src/compiler.rs
// Licensed under the MIT license of Manuel Cerón.

#[derive(PartialOrd, PartialEq, Copy, Clone)]
enum ThePrecedence {
    None,
    Assignment, // =
    Or,         // or
    And,        // and
    Equality,   // == !=
    Comparison, // < > <= >=
    Term,       // + -
    Factor,     // * /
    Unary,      // ! -
    Call,       // . ()
    Primary,
}

impl ThePrecedence {
    fn next_higher(&self) -> ThePrecedence {
        match self {
            ThePrecedence::None => ThePrecedence::Assignment,
            ThePrecedence::Assignment => ThePrecedence::Or,
            ThePrecedence::Or => ThePrecedence::And,
            ThePrecedence::And => ThePrecedence::Equality,
            ThePrecedence::Equality => ThePrecedence::Comparison,
            ThePrecedence::Comparison => ThePrecedence::Term,
            ThePrecedence::Term => ThePrecedence::Factor,
            ThePrecedence::Factor => ThePrecedence::Unary,
            ThePrecedence::Unary => ThePrecedence::Call,
            ThePrecedence::Call => ThePrecedence::Primary,
            ThePrecedence::Primary => ThePrecedence::None,
        }
    }
}

type ParseFn = fn(&mut TheCompiler, can_assign: bool) -> ();

#[derive(Copy, Clone)]
struct TheParseRule {
    prefix: Option<ParseFn>,
    infix: Option<ParseFn>,
    precedence: ThePrecedence,
}

impl TheParseRule {
    fn new(
        prefix: Option<ParseFn>,
        infix: Option<ParseFn>,
        precedence: ThePrecedence,
    ) -> TheParseRule {
        TheParseRule {
            prefix,
            infix,
            precedence,
        }
    }
}

#[derive(Clone, Debug)]
pub struct TheCompilerError {
    // This stack is only used for verification during compilation.
    pub location: (u32, u32),
    pub message: String,
}

impl TheCompilerError {
    pub fn new(location: (u32, u32), message: String) -> Self {
        Self { location, message }
    }
}

#[derive(Clone, Debug)]
pub struct TheCompilerContext {
    // This stack is only used for verification during compilation.
    pub stack: Vec<TheValue>,
    pub local: Vec<TheCodeObject>,
    pub location: (u32, u32),

    pub module: TheCodeModule,
    pub functions: Vec<TheCodeFunction>,
    pub curr_function_index: usize,

    pub error: Option<TheCompilerError>,
}

impl Default for TheCompilerContext {
    fn default() -> Self {
        TheCompilerContext::new()
    }
}

impl TheCompilerContext {
    pub fn new() -> Self {
        Self {
            stack: vec![],
            local: vec![TheCodeObject::default()],
            location: (0, 0),

            module: TheCodeModule::default(),
            functions: vec![TheCodeFunction::default()],
            curr_function_index: 0,

            error: None,
        }
    }

    /// Returns the current function.
    pub fn get_current_function(&mut self) -> &mut TheCodeFunction {
        &mut self.functions[self.curr_function_index]
    }

    /// Add a function.
    pub fn add_function(&mut self, function: TheCodeFunction) {
        self.functions.push(function);
        self.curr_function_index += 1;
    }
}

pub struct TheCompiler {
    rules: FxHashMap<TheAtomKind, TheParseRule>,
    grid: TheCodeGrid,

    ctx: TheCompilerContext,

    current: TheAtom,
    previous: TheAtom,
}

impl Default for TheCompiler {
    fn default() -> Self {
        TheCompiler::new()
    }
}

impl TheCompiler {
    pub fn new() -> Self {
        let mut rules = FxHashMap::default();

        let mut rule = |kind, prefix, infix, precedence| {
            rules.insert(kind, TheParseRule::new(prefix, infix, precedence));
        };

        use TheAtomKind::*;
        use ThePrecedence as P;

        rule(Number, Some(TheCompiler::number), None, P::None);
        rule(Plus, None, Some(TheCompiler::binary), P::Term);
        rule(Star, None, Some(TheCompiler::binary), P::Factor);
        rule(Eof, None, None, P::None);

        Self {
            rules,
            grid: TheCodeGrid::default(),

            ctx: TheCompilerContext::default(),

            current: TheAtom::End,
            previous: TheAtom::End,
        }
    }

    pub fn compile(&mut self, grid: &mut TheCodeGrid) -> Result<TheCodeModule, TheCompilerError> {
        self.current = TheAtom::End;
        self.previous = TheAtom::End;

        self.ctx = TheCompilerContext::default();

        grid.clear_messages();
        self.grid = grid.clone();

        self.advance();

        while !self.matches(TheAtomKind::Eof) && self.ctx.error.is_none() {
            self.declaration();
        }

        if let Some(error) = &self.ctx.error {
            println!("Error: {:?}", error);
            grid.add_message(
                error.location,
                TheCodeGridMessage {
                    message_type: TheCodeGridMessageType::Error,
                    message: error.message.clone(),
                },
            );
            Err(error.clone())
        } else {

            if !self.ctx.get_current_function().is_empty() {
                let f = self.ctx.get_current_function().clone();
                self.ctx.module.insert_function(f.name.clone(), f);
            }

            println!("End:{:?}", self.ctx.functions);
            Ok(self.ctx.module.clone())
        }
    }

    fn declaration(&mut self) {
        if self.current.can_assign() {
            // Local or Object variable
            self.advance();
            let var = self.previous.clone();
            self.var_declaration();
            let node = var.to_node(&mut self.ctx);
            self.ctx.get_current_function().add_node(node);
        }
    }

    fn var_declaration(&mut self) {
        self.expression();
    }

    fn expression(&mut self) {
        self.parse_precedence(ThePrecedence::Assignment);
    }

    fn number(&mut self, _can_assign: bool) {
        let node = self.previous.to_node(&mut self.ctx);
        self.ctx.get_current_function().add_node(node);
    }

    fn binary(&mut self, _can_assign: bool) {
        let operator_type = self.previous.to_kind();

        let rule = self.get_rule(operator_type);
        self.parse_precedence(rule.precedence.next_higher());

        match operator_type {
            // TokenType::BangEqual => self.emit_instructions(Instruction::Equal, Instruction::Not),
            // TokenType::EqualEqual => self.emit_instruction(Instruction::Equal),
            // TokenType::Greater => self.emit_instruction(Instruction::Greater),
            // TokenType::GreaterEqual => self.emit_instructions(Instruction::Less, Instruction::Not),
            // TokenType::Less => self.emit_instruction(Instruction::Less),
            // TokenType::LessEqual => self.emit_instructions(Instruction::Greater, Instruction::Not),
            TheAtomKind::Plus => {
                let node = TheAtom::Add().to_node(&mut self.ctx);
                self.ctx.get_current_function().add_node(node);
            }
            TheAtomKind::Star => {
                let node = TheAtom::Multiply().to_node(&mut self.ctx);
                self.ctx.get_current_function().add_node(node);
            }
            // TokenType::Minus => self.emit_instruction(Instruction::Subtract),
            // TokenType::Star => self.emit_instruction(Instruction::Multiply),
            // TokenType::Slash => self.emit_instruction(Instruction::Divide),
            _ => {}
        }
    }

    fn get_rule(&self, kind: TheAtomKind) -> TheParseRule {
        self.rules.get(&kind).cloned().unwrap()
    }

    fn parse_precedence(&mut self, precedence: ThePrecedence) {
        self.advance();

        let prefix_rule = self.get_rule(self.previous.to_kind()).prefix;
        let can_assign = precedence <= ThePrecedence::Assignment;

        if let Some(prefix_rule) = prefix_rule {
            prefix_rule(self, can_assign);
        } else {
            //self.error("Expect expression.");
            return;
        }

        while precedence <= self.get_rule(self.current.to_kind()).precedence {
            self.advance();
            let infix_rule = self.get_rule(self.previous.to_kind()).infix;

            if let Some(infix_rule) = infix_rule {
                infix_rule(self, can_assign);
            }
        }

        if can_assign && self.matches(TheAtomKind::Equal) {
            //self.error("Invalid assignment target.");
        }
    }

    /// Advance one token
    fn advance(&mut self) {
        self.previous = self.current.clone();

        if let Some(location) = self.grid.current_pos {
            self.ctx.location = location;
        }

        self.current = self.grid.get_next(false);
        println!("{:?} : {:?}", self.grid.current_pos, self.current);

        /*
        loop {
            self.parser.current = if self.code.is_empty() {
                TheAtom::Stop
            } else {
                self.code.remove(0)
            };

            if self.parser.current.to_kind() != TheAtomKind::Error {
                break;
            }
            //self.error_at_current(self.parser.current.lexeme.clone().as_str());
        }*/
    }

    fn matches(&mut self, kind: TheAtomKind) -> bool {
        if !self.check(kind) {
            false
        } else {
            self.advance();
            true
        }
    }

    fn check(&self, kind: TheAtomKind) -> bool {
        self.current.to_kind() == kind
    }

    /*
    /// Error at the current token
    fn error_at_current(&mut self, message: &str) {
        self.error_at(self.parser.current.clone(), message)
    }

    /// Error at the previous token
    fn error(&mut self, message: &str) {
        self.error_at(self.parser.previous.clone(), message)
    }

    /// Error at the given token
    fn error_at(&mut self, _token: TheAtom, message: &str) {
        println!("error {}", message);
        if self.parser.panic_mode {
            return;
        }
        self.parser.panic_mode = true;
        self.parser.error_message = message.to_owned();
        //self.parser.error_line = self.parser.previous.line;
        self.parser.had_error = true;
    }*/
}
