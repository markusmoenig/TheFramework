use crate::prelude::*;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum TheAtom {
    Value(TheValue),
    Add(),
    Multiply(),
    LocalGet(String),
    LocalSet(String),
    End,
}

impl TheAtom {

    pub fn can_assign(&self) -> bool {
        matches!(self, TheAtom::LocalSet(_name))
    }

    pub fn to_node(&self, ctx: &mut TheCompilerContext) -> TheExeNode {
        match self {
            TheAtom::End => {
                let call: TheExeNodeCall = |_stack: &mut Vec<TheValue>, _values: &Vec<TheValue>, _env: &mut TheExeEnvironment| {};
                TheExeNode::new(call, vec![])
            }
            TheAtom::LocalGet(name) => {
                let call: TheExeNodeCall = |_stack: &mut Vec<TheValue>, _values: &Vec<TheValue>, _env: &mut TheExeEnvironment| {};
                TheExeNode::new(call, vec![])
            }
            TheAtom::LocalSet(name) => {
                let call: TheExeNodeCall = |stack: &mut Vec<TheValue>, values: &Vec<TheValue>, env: &mut TheExeEnvironment| {
                    if let Some(local) = env.local.last_mut() {
                        local.set(values[0].to_string().unwrap(), stack.pop().unwrap())
                    }
                };

                if ctx.stack.is_empty() {
                    ctx.error = Some(TheCompilerError::new(
                        ctx.location,
                        "Nothing to assign to local variable.".to_string()
                    ));
                } else if let Some(local) = ctx.local.last_mut() {
                    local.set(name.clone(), ctx.stack.pop().unwrap());
                }

                TheExeNode::new(call, vec![TheValue::Text(name.clone())])
            }
            TheAtom::Value(value) => {
                let call: TheExeNodeCall = |stack: &mut Vec<TheValue>, values: &Vec<TheValue>, _env: &mut TheExeEnvironment| {
                    stack.push(values[0].clone());
                };

                ctx.stack.push(value.clone());

                TheExeNode::new(call, vec![value.clone()])
            }
            TheAtom::Add() => {
                let call: TheExeNodeCall = |stack: &mut Vec<TheValue>, _values: &Vec<TheValue>, _env: &mut TheExeEnvironment| {
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

                TheExeNode::new(call, vec![])
            }
            TheAtom::Multiply() => {
                let call: TheExeNodeCall = |stack: &mut Vec<TheValue>, _values: &Vec<TheValue>, _env: &mut TheExeEnvironment| {
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

                TheExeNode::new(call, vec![])
            }
        }
    }

    pub fn to_kind(&self) -> TheAtomKind {
        match self {
            TheAtom::End => TheAtomKind::Eof,
            TheAtom::LocalGet(_name) => TheAtomKind::Identifier,
            TheAtom::LocalSet(_name) => TheAtomKind::Identifier,
            TheAtom::Value(_value) => TheAtomKind::Number,
            TheAtom::Add() => TheAtomKind::Plus,
            TheAtom::Multiply() => TheAtomKind::Star,
        }
    }

    pub fn describe(&self) -> String {
        match self {
            TheAtom::End => "Stop".to_string(),
            TheAtom::LocalGet(name) => name.clone(), //"name".to_string(),
            TheAtom::LocalSet(name) => name.clone(), //"name".to_string(),
            TheAtom::Value(value) => value.describe(),
            TheAtom::Add() => "+".to_string(),
            TheAtom::Multiply() => "*".to_string(),
        }
    }

    #[cfg(feature = "ui")]
    /// Generates a text layout to edit the properties of the atom
    pub fn to_layout(&self, layout: &mut dyn TheHLayoutTrait) {
        match self {
            TheAtom::LocalGet(name) => {
                let mut text = TheText::new(TheId::empty());
                text.set_text("Variable Name:".to_string());
                let mut name_edit = TheTextLineEdit::new(TheId::named("Atom Local Get"));
                name_edit.set_text(name.clone());
                name_edit.set_needs_redraw(true);
                layout.add_widget(Box::new(text));
                layout.add_widget(Box::new(name_edit));
            }
            TheAtom::LocalSet(name) => {
                let mut text = TheText::new(TheId::empty());
                text.set_text("Variable Name:".to_string());
                let mut name_edit = TheTextLineEdit::new(TheId::named("Atom Local Set"));
                name_edit.set_text(name.clone());
                name_edit.set_needs_redraw(true);
                layout.add_widget(Box::new(text));
                layout.add_widget(Box::new(name_edit));
            }
            TheAtom::Value(value) => {
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
            TheAtom::Value(_) => {
                //println!("{} {:?}", name, value);
                if name == "Atom Integer Edit" {
                    *self = TheAtom::Value(value.clone());
                }
            }
            _ => {}
        };
    }
}

#[allow(dead_code)]
#[derive(Eq, PartialEq, Debug, Copy, Clone, Hash)]
pub enum TheAtomKind {
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
