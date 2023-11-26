use crate::prelude::*;

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

struct TheParser {
    current: TheAtom,
    previous: TheAtom,

    had_error: bool,
    panic_mode: bool,
    error_message: String,
    error_line: usize,
}

impl TheParser {
    pub fn new() -> Self {
        Self {
            current: TheAtom::Stop,
            previous: TheAtom::Stop,
            had_error: false,
            panic_mode: false,
            error_message: "".to_string(),
            error_line: 0,
        }
    }
}

pub struct TheCompiler {
    parser: TheParser,
    rules: FxHashMap<TheAtomKind, TheParseRule>,
    code: Vec<TheAtom>,
    pipe: TheExePipeline,
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
            parser: TheParser::new(),
            rules,
            code: vec![],
            pipe: TheExePipeline::new(),
        }
    }

    pub fn compile(&mut self, ctx: TheCodeContext) -> Result<TheExePipeline, String> {
        self.pipe = TheExePipeline::new();

        let mut code = vec![];

        let mut x = 0;
        let mut y = 0;

        loop {
            if let Some(atom) = ctx.code.get(&(x, y)) {
                //let node = atom.to_node();
                code.push(atom.clone());

                x += 1;
            } else if x == 0 {
                break;
            } else {
                x = 0;
                y += 1;
            }
        }

        self.code = code;

        self.advance();

        while !self.matches(TheAtomKind::Eof) {
            self.parse_precedence(ThePrecedence::Assignment);
        }


        Ok(self.pipe.clone())
    }

    fn number(&mut self, _can_assign: bool) {
        self.pipe.add(self.parser.previous.to_node());
    }

    fn binary(&mut self, _can_assign: bool) {
        let operator_type = self.parser.previous.to_kind();

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
                self.pipe.add(TheAtom::Add().to_node());
            }
            TheAtomKind::Star => {
                self.pipe.add(TheAtom::Multiply().to_node());
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

        let prefix_rule = self.get_rule(self.parser.previous.to_kind()).prefix;
        let can_assign = precedence <= ThePrecedence::Assignment;

        if let Some(prefix_rule) = prefix_rule {
            prefix_rule(self, can_assign);
        } else {
            //self.error("Expect expression.");
            return;
        }

        while precedence <= self.get_rule(self.parser.current.to_kind()).precedence {
            self.advance();
            let infix_rule = self.get_rule(self.parser.previous.to_kind()).infix;

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
        self.parser.previous = self.parser.current.clone();

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
        }
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
        self.parser.current.to_kind() == kind
    }

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
        if self.parser.panic_mode { return; }
        self.parser.panic_mode = true;
        self.parser.error_message = message.to_owned();
        //self.parser.error_line = self.parser.previous.line;
        self.parser.had_error = true;
    }
}
