use crate::prelude::*;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum TheCodeAtom {
    Value(TheValue),
    Add,
    Multiply,
    LocalGet(String),
    LocalSet(String),
    FuncDef(String),
    FuncCall(String),
    Return,
    EndOfExpression,
    EndOfCode,
}

impl TheCodeAtom {
    pub fn can_assign(&self) -> bool {
        matches!(self, TheCodeAtom::LocalSet(_name))
    }

    pub fn from_json(json: &str) -> Self {
        serde_json::from_str(json).unwrap_or(TheCodeAtom::EndOfCode)
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }

    pub fn to_node(&self, ctx: &mut TheCompilerContext) -> TheCodeNode {
        match self {
            TheCodeAtom::FuncDef(name) => {
                let call: TheCodeNodeCall =
                    |_stack: &mut Vec<TheValue>,
                     _values: &Vec<TheValue>,
                     _sandbox: &mut TheCodeSandbox| {};
                TheCodeNode::new(call, vec![])
            }
            TheCodeAtom::Return => {
                let call: TheCodeNodeCall =
                    |_stack: &mut Vec<TheValue>,
                     _values: &Vec<TheValue>,
                     _sandbox: &mut TheCodeSandbox| {};
                TheCodeNode::new(call, vec![])
            }
            TheCodeAtom::FuncCall(name) => {
                let call: TheCodeNodeCall =
                    |_stack: &mut Vec<TheValue>,
                     _values: &Vec<TheValue>,
                     _sandbox: &mut TheCodeSandbox| {};
                TheCodeNode::new(call, vec![])
            }

            TheCodeAtom::LocalGet(name) => {
                let call: TheCodeNodeCall =
                    |_stack: &mut Vec<TheValue>,
                     _values: &Vec<TheValue>,
                     _sandbox: &mut TheCodeSandbox| {};
                TheCodeNode::new(call, vec![])
            }
            TheCodeAtom::LocalSet(name) => {
                let call: TheCodeNodeCall =
                    |stack: &mut Vec<TheValue>,
                     values: &Vec<TheValue>,
                     sandbox: &mut TheCodeSandbox| {
                        if let Some(function) = sandbox.call_stack.last_mut() {
                            if let Some(local) = function.local.last_mut() {
                                local.set(values[0].to_string().unwrap(), stack.pop().unwrap())
                            }
                        }
                    };

                if ctx.stack.is_empty() {
                    ctx.error = Some(TheCompilerError::new(
                        ctx.location,
                        "Nothing to assign to local variable.".to_string(),
                    ));
                } else if let Some(local) = ctx.local.last_mut() {
                    local.set(name.clone(), ctx.stack.pop().unwrap());
                }

                TheCodeNode::new(call, vec![TheValue::Text(name.clone())])
            }
            TheCodeAtom::Value(value) => {
                let call: TheCodeNodeCall =
                    |stack: &mut Vec<TheValue>,
                     values: &Vec<TheValue>,
                     _sandbox: &mut TheCodeSandbox| {
                        stack.push(values[0].clone());
                    };

                ctx.stack.push(value.clone());

                TheCodeNode::new(call, vec![value.clone()])
            }
            TheCodeAtom::Add => {
                let call: TheCodeNodeCall =
                    |stack: &mut Vec<TheValue>,
                     _values: &Vec<TheValue>,
                     _sandbox: &mut TheCodeSandbox| {
                        let a = stack.pop().unwrap().to_i32().unwrap();
                        let b = stack.pop().unwrap().to_i32().unwrap();
                        stack.push(TheValue::Int(a + b));
                    };

                if ctx.stack.len() < 2 {
                    ctx.error = Some(TheCompilerError::new(
                        ctx.location,
                        format!("Invalid stack for Add ({})", ctx.stack.len()),
                    ));
                }

                TheCodeNode::new(call, vec![])
            }
            TheCodeAtom::Multiply => {
                let call: TheCodeNodeCall =
                    |stack: &mut Vec<TheValue>,
                     _values: &Vec<TheValue>,
                     _sandbox: &mut TheCodeSandbox| {
                        let a = stack.pop().unwrap().to_i32().unwrap();
                        let b = stack.pop().unwrap().to_i32().unwrap();
                        stack.push(TheValue::Int(a * b));
                    };

                if ctx.stack.len() < 2 {
                    ctx.error = Some(TheCompilerError::new(
                        ctx.location,
                        format!("Invalid stack for Multiply ({})", ctx.stack.len()),
                    ));
                }
                TheCodeNode::new(call, vec![])
            }
            TheCodeAtom::EndOfCode | TheCodeAtom::EndOfExpression => {
                let call: TheCodeNodeCall =
                    |_stack: &mut Vec<TheValue>,
                     _values: &Vec<TheValue>,
                     _sandbox: &mut TheCodeSandbox| {};
                TheCodeNode::new(call, vec![])
            }
        }
    }

    pub fn to_kind(&self) -> TheCodeAtomKind {
        match self {
            TheCodeAtom::FuncDef(_name) => TheCodeAtomKind::Fn,
            TheCodeAtom::FuncCall(_name) => TheCodeAtomKind::Fn,
            TheCodeAtom::Return => TheCodeAtomKind::Return,
            TheCodeAtom::LocalGet(_name) => TheCodeAtomKind::Identifier,
            TheCodeAtom::LocalSet(_name) => TheCodeAtomKind::Identifier,
            TheCodeAtom::Value(_value) => TheCodeAtomKind::Number,
            TheCodeAtom::Add => TheCodeAtomKind::Plus,
            TheCodeAtom::Multiply => TheCodeAtomKind::Star,
            TheCodeAtom::EndOfExpression => TheCodeAtomKind::Semicolon,
            TheCodeAtom::EndOfCode => TheCodeAtomKind::Eof,
        }
    }

    pub fn describe(&self) -> String {
        match self {
            TheCodeAtom::FuncDef(name) => name.clone(),
            TheCodeAtom::FuncCall(name) => name.clone(),
            TheCodeAtom::Return => "Return".to_string(),
            TheCodeAtom::LocalGet(name) => name.clone(), //"name".to_string(),
            TheCodeAtom::LocalSet(name) => name.clone(), //"name".to_string(),
            TheCodeAtom::Value(value) => value.describe(),
            TheCodeAtom::Add => "+".to_string(),
            TheCodeAtom::Multiply => "*".to_string(),
            TheCodeAtom::EndOfExpression => ";".to_string(),
            TheCodeAtom::EndOfCode => "Stop".to_string(),
        }
    }

    #[cfg(feature = "ui")]
    /// Generates a text layout to edit the properties of the atom
    pub fn to_layout(&self, layout: &mut dyn TheHLayoutTrait) {
        match self {
            TheCodeAtom::FuncDef(name) => {
                let mut text = TheText::new(TheId::empty());
                text.set_text("Function Name:".to_string());
                let mut name_edit = TheTextLineEdit::new(TheId::named("Atom Func Def"));
                name_edit.set_text(name.clone());
                name_edit.set_needs_redraw(true);
                layout.add_widget(Box::new(text));
                layout.add_widget(Box::new(name_edit));
            }
            TheCodeAtom::FuncCall(name) => {
                let mut text = TheText::new(TheId::empty());
                text.set_text("Function Name:".to_string());
                let mut name_edit = TheTextLineEdit::new(TheId::named("Atom Func Call"));
                name_edit.set_text(name.clone());
                name_edit.set_needs_redraw(true);
                layout.add_widget(Box::new(text));
                layout.add_widget(Box::new(name_edit));
            }
            TheCodeAtom::LocalGet(name) => {
                let mut text = TheText::new(TheId::empty());
                text.set_text("Variable Name:".to_string());
                let mut name_edit = TheTextLineEdit::new(TheId::named("Atom Local Get"));
                name_edit.set_text(name.clone());
                name_edit.set_needs_redraw(true);
                layout.add_widget(Box::new(text));
                layout.add_widget(Box::new(name_edit));
            }
            TheCodeAtom::LocalSet(name) => {
                let mut text = TheText::new(TheId::empty());
                text.set_text("Variable Name:".to_string());
                let mut name_edit = TheTextLineEdit::new(TheId::named("Atom Local Set"));
                name_edit.set_text(name.clone());
                name_edit.set_needs_redraw(true);
                layout.add_widget(Box::new(text));
                layout.add_widget(Box::new(name_edit));
            }
            TheCodeAtom::Value(value) => {
                let mut text = TheText::new(TheId::empty());
                text.set_text("Integer:".to_string());
                let mut name_edit = TheTextLineEdit::new(TheId::named("Atom Integer"));
                name_edit.set_text(value.describe());
                name_edit.set_needs_redraw(true);
                layout.add_widget(Box::new(text));
                layout.add_widget(Box::new(name_edit));
            }
            _ => {}
        };
    }

    #[cfg(feature = "ui")]
    /// Generates a text layout to edit the properties of the atom
    pub fn process_value_change(&mut self, name: String, value: TheValue) {
        match self {
            TheCodeAtom::Value(_) => {
                //println!("{} {:?}", name, value);
                if name == "Atom Integer Edit" {
                    *self = TheCodeAtom::Value(value.clone());
                }
            }
            _ => {}
        };
    }
}

#[allow(dead_code)]
#[derive(Eq, PartialEq, Debug, Copy, Clone, Hash)]
pub enum TheCodeAtomKind {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    Dollar,
    Colon,

    LineFeed,
    Space,
    Quotation,
    Unknown,
    SingeLineComment,
    HexColor,

    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier,
    String,
    Number,

    // Keywords.
    And,
    Class,
    Else,
    False,
    For,
    Fn,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Let,
    While,
    CodeBlock,

    Error,
    Eof,
}
