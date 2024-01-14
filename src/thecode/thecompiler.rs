use crate::prelude::*;

// Some code taken from https://github.com/ceronman/loxido/blob/master/src/compiler.rs
// Licensed under the MIT license of Manuel Cerón.

#[derive(PartialOrd, PartialEq, Copy, Clone, Debug)]
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

#[derive(Copy, Clone, Debug)]
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
    pub location: (u16, u16),
    pub message: String,
}

impl TheCompilerError {
    pub fn new(location: (u16, u16), message: String) -> Self {
        Self { location, message }
    }
}

#[derive(Clone, Debug)]
pub struct TheCompilerContext {
    // This stack is only used for verification during compilation.
    pub stack: Vec<TheValue>,
    pub local: Vec<TheCodeObject>,

    pub previous_location: (u16, u16),
    pub current_location: (u16, u16),
    pub node_location: (u16, u16),

    pub blocks: Vec<TheCodeNode>,

    pub current: TheCodeAtom,
    pub previous: TheCodeAtom,

    pub module: TheCodeModule,
    pub functions: Vec<TheCodeFunction>,
    pub curr_function_index: usize,

    pub error: Option<TheCompilerError>,
    pub external_call: Option<(TheCodeNodeCall, Vec<TheValue>)>,
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

            blocks: vec![],

            previous_location: (0, 0),
            current_location: (0, 0),
            node_location: (0, 0),

            current: TheCodeAtom::EndOfCode,
            previous: TheCodeAtom::EndOfCode,

            module: TheCodeModule::default(),
            functions: vec![TheCodeFunction::default()],
            curr_function_index: 0,

            error: None,
            external_call: None,
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

    /// Removes the last function from the stack and returns it.
    pub fn remove_function(&mut self) -> Option<TheCodeFunction> {
        if self.curr_function_index > 0 {
            self.curr_function_index -= 1
        }
        self.functions.pop()
    }
}

#[derive(Clone, Debug)]
pub struct TheCompiler {
    rules: FxHashMap<TheCodeAtomKind, TheParseRule>,
    grid: TheCodeGrid,

    external_call: FxHashMap<String, (TheCodeNodeCall, Vec<TheValue>)>,

    ctx: TheCompilerContext,
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

        use TheCodeAtomKind::*;
        use ThePrecedence as P;

        rule(Number, Some(TheCompiler::number), None, P::None);
        rule(Plus, None, Some(TheCompiler::binary), P::Term);
        rule(Star, None, Some(TheCompiler::binary), P::Factor);
        rule(Eof, None, None, P::None);
        rule(Return, None, None, P::None);
        rule(Semicolon, None, None, P::None);
        rule(Identifier, Some(TheCompiler::variable), None, P::None);

        Self {
            rules,
            grid: TheCodeGrid::default(),
            external_call: FxHashMap::default(),
            ctx: TheCompilerContext::default(),
        }
    }

    /// Add an external node to the compiler.
    pub fn add_external_call(&mut self, name: String, call: TheCodeNodeCall, values: Vec<TheValue>) {
        self.external_call.insert(name, (call, values));
    }

    /// Compile the given code grid and returns either a module or an error.
    pub fn compile(&mut self, grid: &mut TheCodeGrid) -> Result<TheCodeModule, TheCompilerError> {
        self.ctx = TheCompilerContext::default();

        grid.clear_messages();
        self.grid = grid.clone();

        self.advance();

        while !self.matches(TheCodeAtomKind::Eof) && self.ctx.error.is_none() {
            if self.ctx.current_location.1 % 2 == 1 {
                self.advance();
                continue;
            }

            let indent = self.ctx.blocks.len();
            if indent > 0
                && self.ctx.current_location.0 == 0
                && self.ctx.current_location.1 > self.ctx.previous_location.1
            {
                // We are at the start of a new line, check if we have a block to close.
                #[allow(clippy::collapsible_if)]
                for has_code_index in 0..indent {
                    if self
                        .grid
                        .code
                        .contains_key(&(has_code_index as u16, self.ctx.current_location.1))
                    {
                        if has_code_index % 2 == 0 {
                            if has_code_index / 2 == indent - 1 {
                                // Closing the block.
                                if let Some(function) = self.ctx.remove_function() {
                                    if let Some(mut node) = self.ctx.blocks.pop() {
                                        node.data.sub_functions.push(function);
                                        self.ctx.get_current_function().add_node(node);
                                    }
                                }
                            }
                        }
                    }
                }

                //println!("previous {:?} current {:?}", self.ctx.previous_location, self.ctx.current_location);
            }

            self.declaration();
        }

        let indent = self.ctx.blocks.len();
        if indent == 1 {
            if let Some(function) = self.ctx.remove_function() {
                if let Some(mut node) = self.ctx.blocks.pop() {
                    node.data.sub_functions.push(function);
                    self.ctx.get_current_function().add_node(node);
                }
            } else {
                // TODO ERROR MESSAGE: Too many open blocks at the end of the code.
            }
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

            self.ctx.module.codegrid_id = grid.id;
            Ok(self.ctx.module.clone())
        }
    }

    fn declaration(&mut self) {
        //println!("declaration {:?}", self.ctx.current);

        match self.ctx.current.clone() {
            TheCodeAtom::FuncDef(name) => {
                self.advance();
                let mut func = TheCodeFunction::named(name);
                let mut arguments = vec![];
                while let TheCodeAtom::FuncArg(arg_name) = self.ctx.current.clone() {
                    if let Some(local) = self.ctx.local.last_mut() {
                        local.set(arg_name.clone(), TheValue::Int(0));
                    }
                    arguments.push(arg_name.clone());
                    self.advance();
                }
                func.arguments = arguments;
                self.ctx.add_function(func);
            }
            TheCodeAtom::LocalSet(_name) => {
                self.advance();
                let var = self.ctx.previous.clone();
                let location = self.ctx.previous_location;

                match &self.ctx.current {
                    TheCodeAtom::Assignment(_op) => {
                        self.advance();
                    }
                    _ => {
                        self.error_at(
                            (
                                self.ctx.previous_location.0 + 1,
                                self.ctx.previous_location.1,
                            ),
                            "Expected assignment operator.",
                        );
                        return;
                    }
                }

                self.expression();
                self.ctx.node_location = location;
                if let Some(node) = var.to_node(&mut self.ctx) {
                    self.ctx.get_current_function().add_node(node);
                }
            }
            TheCodeAtom::ObjectSet(_object, _name) => {
                self.advance();
                let var = self.ctx.previous.clone();
                let location = self.ctx.previous_location;

                match &self.ctx.current {
                    TheCodeAtom::Assignment(_op) => {
                        self.advance();
                    }
                    _ => {
                        self.error_at(
                            (
                                self.ctx.previous_location.0 + 1,
                                self.ctx.previous_location.1,
                            ),
                            "Expected assignment operator.",
                        );
                        return;
                    }
                }

                self.expression();
                self.ctx.node_location = location;
                if let Some(node) = var.to_node(&mut self.ctx) {
                    self.ctx.get_current_function().add_node(node);
                }
            }
            _ => {
                self.statement();
            }
        }
    }

    fn statement(&mut self) {
        match self.ctx.current.clone() {
            TheCodeAtom::Return => {
                self.advance();

                if self
                    .grid
                    .code
                    .contains_key(&(self.ctx.current_location.0 + 1, self.ctx.current_location.1))
                {
                    // This return statement has a value parse if first.
                    let ret = self.ctx.previous.clone();
                    self.expression();

                    if let Some(node) = ret.to_node(&mut self.ctx) {
                        self.ctx.get_current_function().add_node(node);
                    }
                }

                if let Some(f) = self.ctx.remove_function() {
                    self.ctx.module.insert_function(f.name.clone(), f);
                } else {
                    self.error_at_current("Unexpected 'Return' code.")
                }
            }
            TheCodeAtom::FuncCall(_) => {
                self.ctx.node_location = self.ctx.current_location;
                if let Some(node) = self.ctx.current.clone().to_node(&mut self.ctx) {
                    self.ctx.get_current_function().add_node(node);
                }
                self.advance();
            }
            TheCodeAtom::ExternalCall(_, _, _, arg_values, _) => {
                self.advance();
                let external_call = self.ctx.previous.clone();
                let location: (u16, u16) = self.ctx.previous_location;
                self.ctx.node_location = location;

                for (index, _) in arg_values.iter().enumerate() {
                    let off = location.0 + (index + 1) as u16 * 2;

                    if !matches!(self.ctx.current, TheCodeAtom::EndOfExpression) {
                        self.error_at(
                            (
                                self.ctx.current_location.0,
                                self.ctx.current_location.1,
                            ),
                            "Unexpected code inside function call.",
                        );
                        return;
                    }

                    self.advance();
                    //println!("off {:?} loc {:?}", off, self.ctx.current_location);

                    // Check if function argument value at the right position.
                    if self.ctx.current_location.0 != off || self.ctx.current_location.1 != location.1 {
                        self.error_at(
                            (
                                off,
                                location.1,
                            ),
                            "Expected value at this position.",
                        );
                        return;
                    }

                    match &self.ctx.current {
                        TheCodeAtom::Value(_op) => {
                            // Add the function argument to the stack.
                            if let Some(node) = self.ctx.current.clone().to_node(&mut self.ctx) {
                                self.ctx.get_current_function().add_node(node);
                            }
                        }
                        _ => {
                            self.error_at(
                                (
                                    self.ctx.current_location.0,
                                    self.ctx.current_location.1,
                                ),
                                "Expected Value.",
                            );
                            return;
                        }
                    }
                    self.advance();
                }

                if let TheCodeAtom::ExternalCall(name, _, _, _, _) = &external_call {
                    if let Some(call) = self.external_call.get(name) {
                        self.ctx.external_call = Some(call.clone());
                        if let Some(node) = external_call.to_node(&mut self.ctx) {
                            self.ctx.get_current_function().add_node(node);
                        }
                        self.ctx.external_call = None;
                    } else {
                        self.error_at(
                            (
                                location.0,
                                location.1,
                            ),
                            "Unknown external call.",
                        );
                    }
                }
            }
            TheCodeAtom::Value(value) => {
                self.advance();
                let mut comparison = TheCodeAtom::Comparison("==".to_string());
                let location: (u16, u16) = self.ctx.previous_location;

                match &self.ctx.current.clone() {
                    TheCodeAtom::Comparison(op) => {
                        // Write the value to the stack if the next operation is a comparison
                        if let Some(node) = self.ctx.previous.clone().to_node(&mut self.ctx) {
                            self.ctx.get_current_function().add_node(node);
                        }

                        comparison = TheCodeAtom::Comparison(op.clone());
                        self.advance();
                    }
                    _ => {
                        if self.ctx.previous_location.0 == self.ctx.blocks.len() as u16 * 2 {
                            self.error_at(
                                (
                                    self.ctx.previous_location.0 + 1,
                                    self.ctx.previous_location.1,
                                ),
                                "Expected comparison operator.",
                            );
                            return;
                        }
                    }
                }

                // Load the condition value

                let func = TheCodeFunction::default();
                self.ctx.add_function(func);

                self.expression();
                self.ctx.node_location = location;
                // Write the comparison function which will take the current function as a sub.
                if let Some(mut node) = comparison.to_node(&mut self.ctx) {
                    node.data.values[0] = value;

                    //println!("condition start");

                    let func = TheCodeFunction::default();
                    self.ctx.add_function(func);

                    // We indent one
                    self.ctx.blocks.push(node);
                }
            }
            TheCodeAtom::EndOfExpression => {
                self.advance();
            }
            _ => {
                self.advance();
                self.ctx.error = Some(TheCompilerError::new(
                    self.ctx.current_location,
                    "Unexpected code.".to_string(),
                ));
            }
        }
    }

    fn expression(&mut self) {
        self.parse_precedence(ThePrecedence::Assignment);
    }

    fn variable(&mut self, _can_assing: bool) {
        match self.ctx.previous.clone() {
            TheCodeAtom::LocalGet(_name) => {
                if let Some(node) = self.ctx.previous.clone().to_node(&mut self.ctx) {
                    self.ctx.get_current_function().add_node(node);
                }
            }
            TheCodeAtom::ObjectGet(_, _) => {
                if let Some(node) = self.ctx.previous.clone().to_node(&mut self.ctx) {
                    self.ctx.get_current_function().add_node(node);
                }
            }
            TheCodeAtom::FuncCall(_) | TheCodeAtom::ExternalCall(_, _, _, _, _) => {
                let node = self.ctx.previous.clone().to_node(&mut self.ctx);
                //println!("FuncCall {:?}", self.ctx.current_location);

                let arg_loc = (self.ctx.current_location.0, self.ctx.current_location.1 + 1);

                if let Some(arg) = self.grid.code.get(&arg_loc).cloned() {
                    //self.ctx.previous_location = (arg_loc.0 - 1, arg_loc.1);
                    //self.ctx.current_location = arg_loc;
                    //self.grid.current_pos = Some((arg_loc.0 - 1, arg_loc.1));

                    //self.ctx.current = TheCodeAtom::EndOfExpression;
                    //self.ctx.previous = TheCodeAtom::EndOfExpression;

                    //let arg0 = self.ctx.current.clone();
                    // println!("Starting expression");
                    // self.expression();
                    // println!("end expression");
                    //let node = arg0.to_node(&mut self.ctx);
                    //self.ctx.get_current_function().add_node(node);

                    if let Some(arg_node) = arg.clone().to_node(&mut self.ctx) {
                        self.ctx.get_current_function().add_node(arg_node);
                    }

                    self.grid.code.remove(&arg_loc);
                }

                if let Some(node) = node {
                    self.ctx.get_current_function().add_node(node);
                }
            }
            _ => {
                self.error_at_current("Unknown identifier.");
            }
        }
    }

    fn number(&mut self, _can_assign: bool) {
        if let Some(node) = self.ctx.previous.clone().to_node(&mut self.ctx) {
            self.ctx.get_current_function().add_node(node);
        }
    }

    fn binary(&mut self, _can_assign: bool) {
        let operator_type = self.ctx.previous.to_kind();

        let rule = self.get_rule(operator_type);
        self.parse_precedence(rule.precedence.next_higher());

        match operator_type {
            // TokenType::BangEqual => self.emit_instructions(Instruction::Equal, Instruction::Not),
            // TokenType::EqualEqual => self.emit_instruction(Instruction::Equal),
            // TokenType::Greater => self.emit_instruction(Instruction::Greater),
            // TokenType::GreaterEqual => self.emit_instructions(Instruction::Less, Instruction::Not),
            // TokenType::Less => self.emit_instruction(Instruction::Less),
            // TokenType::LessEqual => self.emit_instructions(Instruction::Greater, Instruction::Not),
            TheCodeAtomKind::Plus => {
                if let Some(node) = TheCodeAtom::Add.to_node(&mut self.ctx) {
                    self.ctx.get_current_function().add_node(node);
                }
            }
            TheCodeAtomKind::Star => {
                if let Some(node) = TheCodeAtom::Multiply.to_node(&mut self.ctx) {
                    self.ctx.get_current_function().add_node(node);
                }
            }
            // TokenType::Minus => self.emit_instruction(Instruction::Subtract),
            // TokenType::Star => self.emit_instruction(Instruction::Multiply),
            // TokenType::Slash => self.emit_instruction(Instruction::Divide),
            _ => {}
        }
    }

    fn get_rule(&self, kind: TheCodeAtomKind) -> TheParseRule {
        self.rules.get(&kind).cloned().unwrap()
    }

    fn parse_precedence(&mut self, precedence: ThePrecedence) {
        self.advance();

        let prefix_rule = self.get_rule(self.ctx.previous.to_kind()).prefix;
        let can_assign = precedence <= ThePrecedence::Assignment;

        if let Some(prefix_rule) = prefix_rule {
            prefix_rule(self, can_assign);
        } else {
            //self.error("Expect expression.");
            return;
        }

        while precedence <= self.get_rule(self.ctx.current.to_kind()).precedence {
            if self.ctx.error.is_some() {
                return;
            }

            self.advance();
            let infix_rule = self.get_rule(self.ctx.previous.to_kind()).infix;

            if let Some(infix_rule) = infix_rule {
                infix_rule(self, can_assign);
            }
        }

        if can_assign && self.matches(TheCodeAtomKind::Equal) {
            //self.error("Invalid assignment target.");
        }
    }

    /// Advance one token
    fn advance(&mut self) {
        self.ctx.previous = self.ctx.current.clone();
        self.ctx.previous_location = self.ctx.current_location;

        self.ctx.current = self.grid.get_next(false);

        if let Some(location) = self.grid.current_pos {
            self.ctx.current_location = location;
        }

        //println!("({:?} : {:?}), ({:?} : {:?})", self.ctx.previous, self.ctx.previous_location, self.grid.current_pos, self.ctx.current);
    }

    fn matches(&mut self, kind: TheCodeAtomKind) -> bool {
        if !self.check(kind) {
            false
        } else {
            self.advance();
            true
        }
    }

    fn check(&self, kind: TheCodeAtomKind) -> bool {
        self.ctx.current.to_kind() == kind
    }

    /// Create an error at the current parser location.
    fn error_at_current(&mut self, message: &str) {
        self.ctx.error = Some(TheCompilerError::new(
            self.ctx.current_location,
            message.to_string(),
        ));
    }

    /// Create an error at the given parser location.
    fn error_at(&mut self, location: (u16, u16), message: &str) {
        self.ctx.error = Some(TheCompilerError::new(location, message.to_string()));
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
    fn error_at(&mut self, _token: TheCodeAtom, message: &str) {
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
