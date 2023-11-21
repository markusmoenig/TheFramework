// Some code taken from https://github.com/ceronman/loxido/blob/master/src/compiler.rs
// Licensed under the MIT license of Manuel Cerón.

use crate::prelude::*;

#[derive(PartialOrd, PartialEq, Copy, Clone)]
enum Precedence {
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

impl Precedence {
    fn next_higher(&self) -> Precedence {
        match self {
            Precedence::None => Precedence::Assignment,
            Precedence::Assignment => Precedence::Or,
            Precedence::Or => Precedence::And,
            Precedence::And => Precedence::Equality,
            Precedence::Equality => Precedence::Comparison,
            Precedence::Comparison => Precedence::Term,
            Precedence::Term => Precedence::Factor,
            Precedence::Factor => Precedence::Unary,
            Precedence::Unary => Precedence::Call,
            Precedence::Call => Precedence::Primary,
            Precedence::Primary => Precedence::None,
        }
    }
}

type ParseFn = fn(&mut TheCompiler, can_assing: bool) -> ();

#[derive(Copy, Clone)]
struct ParseRule {
    prefix: Option<ParseFn>,
    infix: Option<ParseFn>,
    precedence: Precedence,
}

impl ParseRule {
    fn new(prefix: Option<ParseFn>, infix: Option<ParseFn>, precedence: Precedence) -> ParseRule {
        ParseRule {
            prefix,
            infix,
            precedence,
        }
    }
}

struct Parser {
    current: Token,
    previous: Token,

    had_error: bool,
    panic_mode: bool,
    error_message: String,
    error_line: usize,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            current: Token::synthetic("".to_owned()),
            previous: Token::synthetic("".to_owned()),
            had_error: false,
            panic_mode: false,
            error_message: "".to_string(),
            error_line: 0,
        }
    }
}

#[derive(Clone)]
pub struct Local {
    name: Token,
    depth: usize,
}

#[derive(Clone)]
pub struct Function {
    enclosing: Box<Option<Function>>,

    locals: [Local; 256],

    local_count: usize,
    scope_depth: usize,

    function: ObjectFunction,
}

impl Function {
    pub fn new(name: String, function_type: FunctionType) -> Self {
        Self {
            enclosing: Box::new(None),
            locals: [(); 256].map(|_| Local {
                name: Token {
                    kind: TokenType::Nil,
                    line: 0,
                    lexeme: "".to_string(),
                    indent: 0,
                },
                depth: 0,
            }),
            local_count: 0,
            scope_depth: 0,

            function: ObjectFunction::new(name, function_type),
        }
    }
}

pub struct TheCompiler {
    parser: Parser,
    scanner: TheScanner,

    rules: FxHashMap<TokenType, ParseRule>,

    current: Function,

    pub objects: Vec<Object>,
    current_object: Option<usize>,
}

impl Default for TheCompiler {
    fn default() -> Self {
        Self::new()
    }
}

impl TheCompiler {
    pub fn new() -> Self {
        let mut rules = FxHashMap::default();

        let mut rule = |kind, prefix, infix, precedence| {
            rules.insert(kind, ParseRule::new(prefix, infix, precedence));
        };

        use Precedence as P;
        use TokenType::*;
        rule(
            LeftParen,
            Some(TheCompiler::grouping),
            Some(TheCompiler::call),
            P::Call,
        );

        rule(RightParen, None, None, P::None);
        rule(LeftBrace, None, None, P::None);
        rule(RightBrace, None, None, P::None);
        rule(Comma, None, None, P::None);
        //rule(Dot, None, Some(Compiler::dot), P::Call);
        rule(
            Minus,
            Some(TheCompiler::unary),
            Some(TheCompiler::binary),
            P::Term,
        );
        rule(Plus, None, Some(TheCompiler::binary), P::Term);
        rule(Semicolon, None, None, P::None);
        rule(Slash, None, Some(TheCompiler::binary), P::Factor);
        rule(Star, None, Some(TheCompiler::binary), P::Factor);
        rule(Bang, Some(TheCompiler::unary), None, P::None);
        rule(BangEqual, None, Some(TheCompiler::binary), P::Equality);
        rule(Equal, None, None, P::None);
        rule(EqualEqual, None, Some(TheCompiler::binary), P::Equality);
        rule(Greater, None, Some(TheCompiler::binary), P::Comparison);
        rule(GreaterEqual, None, Some(TheCompiler::binary), P::Comparison);
        rule(Less, None, Some(TheCompiler::binary), P::Comparison);
        rule(LessEqual, None, Some(TheCompiler::binary), P::Comparison);
        rule(Identifier, Some(TheCompiler::variable), None, P::None);
        rule(String, Some(TheCompiler::string), None, P::None);
        rule(Number, Some(TheCompiler::number), None, P::None);
        rule(And, None, Some(TheCompiler::and_op), P::And);
        rule(Class, None, None, P::None);
        rule(Else, None, None, P::None);
        rule(False, Some(TheCompiler::literal), None, P::None);
        rule(For, None, None, P::None);
        rule(Fn, None, None, P::None);
        rule(If, None, None, P::None);
        rule(Nil, Some(TheCompiler::literal), None, P::None);
        rule(Or, None, Some(TheCompiler::or_op), P::Or);
        rule(Print, None, None, P::None);
        rule(Return, None, None, P::None);
        //rule(Super, Some(Compiler::super_), None, P::None);
        //rule(This, Some(Compiler::this), None, P::None);
        rule(True, Some(TheCompiler::literal), None, P::None);
        //rule(Var, None, None, P::None);
        rule(While, None, None, P::None);
        rule(Error, None, None, P::None);
        rule(Eof, None, None, P::None);

        Self {
            parser: Parser::new(),
            scanner: TheScanner::new("".to_string()),
            rules,
            current: Function::new("script".to_string(), FunctionType::Script),

            objects: vec![],
            current_object: None,
        }
    }

    fn init_function(&mut self, function_type: FunctionType) {
        let mut function = Function::new(self.parser.previous.lexeme.clone(), function_type);
        function.enclosing = Box::new(Some(self.current.clone()));

        self.current = function;
    }

    fn end_function(&mut self) -> ObjectFunction {
        self.emit_return();

        let function = self.current.function.clone();

        let temp = self.current.enclosing.clone();
        self.current = temp.unwrap();
        function
    }

    /// Compile the code into a Chunk
    pub fn compile(&mut self, code: String) -> Result<ObjectFunction, InterpretError> {
        self.scanner = TheScanner::new(code);
        self.parser = Parser::new();

        self.advance();

        while !self.matches(TokenType::Eof) {
            self.declaration();
        }

        if self.parser.had_error {
            return Err(InterpretError::CompileError(
                self.parser.error_message.clone(),
                self.parser.error_line,
            ));
        }

        Ok(self.current.function.clone())
    }

    fn declaration(&mut self) {
        // if self.indent() == 0 {
        //     if self.matches(TokenType::Identifier) {
        //         self.object_declaration();
        //     } else if self.matches(TokenType::Let) {
        //         self.var_declaration();
        //     } else {
        //         println!("{:?}", self.parser.previous.kind);
        //         self.error_at_current("Expected node identifier or 'let'.");
        //         self.advance();
        //         if self.parser.panic_mode {
        //             self.synchronize();
        //         }
        //         return;
        //     }

        if self.matches(TokenType::Class) {
            self.class_declaration();
        } else if self.matches(TokenType::Let) {
            self.var_declaration();
        } else {
            self.statement();
        }

        if self.parser.panic_mode {
            self.synchronize();
        }
    }

    fn class_declaration(&mut self) {
        if self.current_object.is_none() && self.check(TokenType::Less) {
            let node_type_name = self.parser.previous.lexeme.clone();

            self.current_object = Some(self.objects.len());
            self.objects.push(Object::new(node_type_name));

            // Read the properties
            while self.check(TokenType::Less) == true {
                self.advance();
                self.consume(TokenType::Identifier, "Expect property name after '<'.");

                let name = self.parser.previous.lexeme.clone().to_lowercase();
                self.consume(TokenType::Colon, "Expect ':' after property name.");

                let v = self.parser.current.lexeme.clone();
                match self.parser.current.kind {
                    TokenType::String => self
                        .gcn()
                        .unwrap()
                        .add_property(name.clone(), Value::String(v.replace("\"", ""))),
                    _ => {
                        self.error_at_current(
                            format!("Unknown property value for '{}'", name).as_str(),
                        );
                    }
                }
                self.advance();
                self.consume(TokenType::Greater, "Expect '>' after property value.");
            }

            self.consume(
                TokenType::CodeBlock,
                "Expect '--' to start node code block.",
            );
            while
            /*self.indent() > 0 &&*/
            !self.check(TokenType::CodeBlock) && !self.check(TokenType::Eof) {
                self.declaration();
            }
            self.consume(TokenType::CodeBlock, "Expect '--' after node code block.");
        }
        self.current_object = None;
    }

    fn gcn(&mut self) -> Option<&mut Object> {
        if self.current_object.is_some() {
            return Some(&mut self.objects[self.current_object.unwrap()]);
        }
        None
    }

    fn fn_declaration(&mut self) {
        let global = self.parse_variable("Expect function name.");
        self.mark_initialized();

        self.function();
        self.define_variable(global);
    }

    fn function(&mut self) {
        self.init_function(FunctionType::Function);

        self.begin_scope();

        self.consume(TokenType::LeftParen, "Expect '(' after function name.");

        if self.check(TokenType::RightParen) == false {
            loop {
                self.current.function.arity += 1;

                let constant = self.parse_variable("Expect parameter name.");
                self.define_variable(constant);
                if self.matches(TokenType::Comma) == false {
                    break;
                }
            }
        }

        self.consume(
            TokenType::RightParen,
            "Expect ')' after function parameters.",
        );
        self.consume(TokenType::LeftBrace, "Expect '{' before function body.");

        self.block();

        let function = self.end_function();
        let instruction = self.make_constant(Value::Function(function));
        self.emit_instruction(instruction);
    }

    fn var_declaration(&mut self) {
        let index = self.parse_variable("Expect variable name.");
        if self.matches(TokenType::Equal) {
            self.expression();
        } else {
            self.emit_instruction(TheInstruction::Nil);
        }
        self.consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration.",
        );
        self.define_variable(index);
    }

    fn parse_variable(&mut self, msg: &str) -> TheInstruction {
        self.consume(TokenType::Identifier, msg);

        self.declare_variable();
        if self.current.scope_depth > 0 {
            return TheInstruction::Nil;
        }

        self.identifier_constant(self.parser.previous.clone())
    }

    fn define_variable(&mut self, index: TheInstruction) {
        if self.current.scope_depth > 0 {
            self.mark_initialized();
            return;
        }
        match index {
            TheInstruction::Constant(offset) => {
                self.emit_instruction(TheInstruction::DefineGlobal(offset))
            }
            _ => {}
        }
    }

    fn mark_initialized(&mut self) {
        if self.current.scope_depth == 0 {
            return;
        }
        self.current.locals[self.current.local_count - 1].depth = self.current.scope_depth;
    }

    fn declare_variable(&mut self) {
        if self.current.scope_depth == 0 {
            return;
        }

        self.add_local(self.parser.previous.clone());
    }

    fn add_local(&mut self, token: Token) {
        if self.current.local_count == 256 {
            self.error("Too many local variables in function.");
            return;
        }
        let local = Local {
            name: token,
            depth: self.current.scope_depth,
        };
        self.current.locals[self.current.local_count] = local;
        self.current.local_count += 1;
    }

    fn resolve_local(&mut self, name: &str) -> Option<usize> {
        for counter in (0..self.current.local_count).rev() {
            if self.current.locals[counter].name.lexeme == name {
                return Some(counter);
            }
        }
        None
    }

    fn statement(&mut self) {
        if self.matches(TokenType::Print) {
            self.print_statement();
        } else if self.matches(TokenType::For) {
            self.for_statement();
        } else if self.matches(TokenType::If) {
            self.if_statement();
        } else if self.matches(TokenType::Return) {
            self.return_statement();
        } else if self.matches(TokenType::While) {
            self.while_statement();
        } else if self.matches(TokenType::LeftBrace) {
            self.begin_scope();
            self.block();
            self.end_scope();
        } else {
            self.expression_statement();
        }
    }

    fn return_statement(&mut self) {
        if self.matches(TokenType::Semicolon) {
            self.emit_return();
        } else {
            self.expression();
            self.consume(TokenType::Semicolon, "Expect ';' after loop return value.");
            self.emit_instruction(TheInstruction::Return);
        }
    }

    fn for_statement(&mut self) {
        self.begin_scope();
        self.consume(TokenType::LeftParen, "Expect '(' after 'for'.");

        // Initializer
        if self.matches(TokenType::Semicolon) {
            // no initializer
        } else if self.matches(TokenType::Let) {
            self.var_declaration();
        } else {
            self.expression_statement();
        }

        let mut loop_start = self.current_chunk().count;

        // Condition
        let mut exit_jump = None;
        if !self.matches(TokenType::Semicolon) {
            self.expression();
            self.consume(TokenType::Semicolon, "Expect ';' after loop condition.");
            let jump = self.emit_jump(TheInstruction::JumpIfFalse(0));
            exit_jump = Some(jump);
            self.emit_instruction(TheInstruction::Pop);
        }

        // Increment
        if !self.matches(TokenType::RightParen) {
            let body_jump = self.emit_jump(TheInstruction::Jump(0));
            let increment_start = self.current_chunk().count;
            self.expression();
            self.emit_instruction(TheInstruction::Pop);
            self.consume(TokenType::RightParen, "Expect ')' after for clauses.");
            self.emit_loop(loop_start);
            loop_start = increment_start;
            self.patch_jump(body_jump);
        }
        self.statement();
        self.emit_loop(loop_start);
        if let Option::Some(exit_jump) = exit_jump {
            self.patch_jump_if_false(exit_jump);
            self.emit_instruction(TheInstruction::Pop);
        }

        self.end_scope();
    }

    fn while_statement(&mut self) {
        let loop_start = self.current_chunk().count;
        self.consume(TokenType::LeftParen, "Expect '(' after 'while'.");
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after condition.");

        let exit_jump = self.emit_jump(TheInstruction::JumpIfFalse(0));
        self.emit_instruction(TheInstruction::Pop);
        self.statement();

        self.emit_loop(loop_start);

        self.patch_jump_if_false(exit_jump);
        self.emit_instruction(TheInstruction::Pop);
    }

    fn if_statement(&mut self) {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.");
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after condition.");

        let then_jump = self.emit_jump(TheInstruction::JumpIfFalse(0));
        self.emit_instruction(TheInstruction::Pop);
        self.statement();
        let else_jump = self.emit_jump(TheInstruction::Jump(0));
        self.patch_jump_if_false(then_jump);
        self.emit_instruction(TheInstruction::Pop);

        if self.matches(TokenType::Else) {
            self.statement();
        }
        self.patch_jump(else_jump);
    }

    fn or_op(&mut self, _can_assign: bool) {
        let else_jump = self.emit_jump(TheInstruction::JumpIfFalse(0));
        let end_jump = self.emit_jump(TheInstruction::Jump(0));

        self.patch_jump_if_false(else_jump);
        self.emit_instruction(TheInstruction::Pop);

        self.parse_precedence(Precedence::Or);
        self.patch_jump(end_jump);
    }

    fn call(&mut self, _can_assign: bool) {
        let mut arg_count = 0;
        if self.check(TokenType::RightParen) == false {
            loop {
                self.expression();
                arg_count += 1;

                if self.matches(TokenType::Comma) == false {
                    break;
                }
            }
        }
        self.consume(TokenType::RightParen, "Expect ')' after arguments.");
        self.emit_instruction(TheInstruction::Call(arg_count));
    }

    fn and_op(&mut self, _can_assign: bool) {
        let end_jump = self.emit_jump(TheInstruction::JumpIfFalse(0));
        self.emit_instruction(TheInstruction::Pop);
        self.parse_precedence(Precedence::And);
        self.patch_jump_if_false(end_jump);
    }

    fn emit_loop(&mut self, offset: usize) {
        let jump = self.current_chunk().count - offset + 1;
        self.emit_instruction(TheInstruction::Loop(jump));
    }

    fn emit_jump(&mut self, instruction: TheInstruction) -> usize {
        self.emit_instruction(instruction);
        self.current_chunk().count - 1
    }

    fn patch_jump(&mut self, offset: usize) {
        let jump = self.current_chunk().count - offset - 1;
        self.current_chunk().code[offset] = TheInstruction::Jump(jump);
    }

    fn patch_jump_if_false(&mut self, offset: usize) {
        let jump = self.current_chunk().count - offset - 1;
        self.current_chunk().code[offset] = TheInstruction::JumpIfFalse(jump);
    }

    fn block(&mut self) {
        while !self.check(TokenType::RightBrace) && !self.check(TokenType::Eof) {
            self.declaration();
        }
        self.consume(TokenType::RightBrace, "Expect '}' after block.");
    }

    fn begin_scope(&mut self) {
        self.current.scope_depth += 1;
    }

    fn end_scope(&mut self) {
        self.current.scope_depth -= 1;
        while self.current.local_count > 0
            && self.current.locals[self.current.local_count - 1].depth > self.current.scope_depth
        {
            self.emit_instruction(TheInstruction::Pop);
            self.current.local_count -= 1;
        }
    }

    fn print_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after value.");
        self.emit_instruction(TheInstruction::Print);
    }

    fn expression_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after expression.");
        self.emit_instruction(TheInstruction::Pop);
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();

        let prefix_rule = self.get_rule(self.parser.previous.kind).prefix;

        let can_assign = precedence <= Precedence::Assignment;

        if let Some(prefix_rule) = prefix_rule {
            prefix_rule(self, can_assign);
        } else {
            self.error("Expect expression.");
            return;
        }

        while precedence <= self.get_rule(self.parser.current.kind).precedence {
            self.advance();
            let infix_rule = self.get_rule(self.parser.previous.kind).infix;

            if let Some(infix_rule) = infix_rule {
                infix_rule(self, can_assign);
            }
        }

        if can_assign && self.matches(TokenType::Equal) {
            self.error("Invalid assignment target.");
        }
    }

    fn variable(&mut self, can_assign: bool) {
        self.named_variable(self.parser.previous.clone(), can_assign);
    }

    fn named_variable(&mut self, name: Token, can_assign: bool) {
        if let Some(offset) = self.resolve_local(name.lexeme.as_str()) {
            if can_assign && self.matches(TokenType::Equal) {
                self.expression();
                self.emit_instruction(TheInstruction::SetLocal(offset));
            } else {
                self.emit_instruction(TheInstruction::GetLocal(offset));
            }
            return;
        }
        let index = self.identifier_constant(name.clone());
        if can_assign && self.matches(TokenType::Equal) {
            self.expression();
            match index {
                TheInstruction::Constant(offset) => {
                    self.emit_instruction(TheInstruction::SetGlobal(offset))
                }
                _ => {}
            }
        } else {
            match index {
                TheInstruction::Constant(offset) => {
                    self.emit_instruction(TheInstruction::GetGlobal(offset))
                }
                _ => {}
            }
        }
    }

    fn binary(&mut self, _can_assign: bool) {
        let operator_type = self.parser.previous.kind;

        let rule = self.get_rule(operator_type);
        self.parse_precedence(rule.precedence.next_higher());

        match operator_type {
            TokenType::BangEqual => {
                self.emit_instructions(TheInstruction::Equal, TheInstruction::Not)
            }
            TokenType::EqualEqual => self.emit_instruction(TheInstruction::Equal),
            TokenType::Greater => self.emit_instruction(TheInstruction::Greater),
            TokenType::GreaterEqual => {
                self.emit_instructions(TheInstruction::Less, TheInstruction::Not)
            }
            TokenType::Less => self.emit_instruction(TheInstruction::Less),
            TokenType::LessEqual => {
                self.emit_instructions(TheInstruction::Greater, TheInstruction::Not)
            }
            TokenType::Plus => self.emit_instruction(TheInstruction::Add),
            TokenType::Minus => self.emit_instruction(TheInstruction::Subtract),
            TokenType::Star => self.emit_instruction(TheInstruction::Multiply),
            TokenType::Slash => self.emit_instruction(TheInstruction::Divide),
            _ => {}
        }
    }

    /// Read a grouping ()
    fn unary(&mut self, _can_assign: bool) {
        let operator_type = self.parser.previous.kind;
        self.parse_precedence(Precedence::Unary);
        match operator_type {
            TokenType::Bang => self.emit_instruction(TheInstruction::Not),
            TokenType::Minus => self.emit_instruction(TheInstruction::Negate),
            _ => return,
        }
    }

    /// True, False, Nil
    fn literal(&mut self, _can_assign: bool) {
        match self.parser.previous.kind {
            TokenType::Nil => self.emit_instruction(TheInstruction::Nil),
            TokenType::False => self.emit_instruction(TheInstruction::False),
            TokenType::True => self.emit_instruction(TheInstruction::True),
            _ => return,
        }
    }

    /// Read a grouping ()
    fn grouping(&mut self, _can_assign: bool) {
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after expression.");
    }

    /// Read an expression
    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    /// Adds a string to the current chunk
    fn string(&mut self, _can_assign: bool) {
        let value = self.parser.previous.lexeme.clone().replace("\"", "");
        self.emit_constant(Value::String(value));
    }

    /// Adds a number to the current chunk
    fn number(&mut self, _can_assign: bool) {
        let value = self.parser.previous.lexeme.parse::<f32>().unwrap();
        self.emit_constant(Value::Float(value));
    }

    /// Adds a constant offset instruction to the current chunk
    fn emit_constant(&mut self, value: Value) {
        let instruction = self.make_constant(value);
        self.emit_instruction(instruction);
    }

    /// Generate a string constant from the token lexeme
    fn identifier_constant(&mut self, name: Token) -> TheInstruction {
        self.make_constant(Value::String(name.lexeme))
    }

    /// Adds a constant and returns a const offset value
    fn make_constant(&mut self, value: Value) -> TheInstruction {
        TheInstruction::Constant(self.current_chunk().add_constant(value))
    }

    /// Consume the current token if the type matches
    fn consume(&mut self, kind: TokenType, message: &str) {
        if self.parser.current.kind == kind {
            self.advance();
            return;
        }
        self.error_at_current(message);
    }

    /// Advance one token
    fn advance(&mut self) {
        self.parser.previous = self.parser.current.clone();

        loop {
            self.parser.current = self.scanner.scan_token();

            if self.parser.current.kind != TokenType::Error {
                break;
            }
            self.error_at_current(self.parser.current.lexeme.clone().as_str());
        }
    }

    fn emit_instruction(&mut self, instruction: TheInstruction) {
        let line = self.parser.previous.line;
        self.current_chunk().write(instruction, line);
    }

    fn emit_instructions(&mut self, instruction1: TheInstruction, instruction2: TheInstruction) {
        self.emit_instruction(instruction1);
        self.emit_instruction(instruction2);
    }

    fn emit_return(&mut self) {
        self.emit_instruction(TheInstruction::Nil);
        self.emit_instruction(TheInstruction::Return);
    }

    fn get_rule(&self, kind: TokenType) -> ParseRule {
        self.rules.get(&kind).cloned().unwrap()
    }

    fn current_chunk(&mut self) -> &mut TheChunk {
        if self.current_object.is_some() {
            &mut self.gcn().unwrap().chunk
        } else {
            &mut self.current.function.chunk
        }
    }

    fn matches(&mut self, kind: TokenType) -> bool {
        if !self.check(kind) {
            false
        } else {
            self.advance();
            true
        }
    }

    fn check(&self, kind: TokenType) -> bool {
        self.parser.current.kind == kind
    }

    fn indent(&self) -> usize {
        self.parser.current.indent
    }

    fn synchronize(&mut self) {
        self.parser.panic_mode = false;

        while self.parser.previous.kind != TokenType::Eof {
            if self.parser.previous.kind == TokenType::Semicolon {
                return;
            }

            match self.parser.current.kind {
                TokenType::Class
                | TokenType::Fn
                | TokenType::Let
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => return,
                _ => (),
            }

            self.advance()
        }
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
    fn error_at(&mut self, _token: Token, message: &str) {
        if self.parser.panic_mode {
            return;
        }
        self.parser.panic_mode = true;
        self.parser.error_message = message.to_owned();
        self.parser.error_line = self.parser.previous.line;
        self.parser.had_error = true;
    }
}
