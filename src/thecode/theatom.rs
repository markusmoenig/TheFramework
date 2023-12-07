use crate::prelude::*;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum TheAtom {
    Value(TheValue),
    Add(),
    Multiply(),
    Variable(String),
    Stop,
}

impl TheAtom {
    pub fn to_node(&self, ctx: &mut TheCompilerContext) -> TheExeNode {
        match self {
            TheAtom::Stop => {
                let call: TheExeNodeCall = |_stack: &mut Vec<TheValue>, _values: &Vec<TheValue>, _env: &TheExeEnvironment| {};
                TheExeNode::new(call, vec![])
            }
            TheAtom::Variable(_name) => {
                let call: TheExeNodeCall = |_stack: &mut Vec<TheValue>, _values: &Vec<TheValue>, _env: &TheExeEnvironment| {};
                TheExeNode::new(call, vec![])
            }
            TheAtom::Value(value) => {
                let call: TheExeNodeCall = |stack: &mut Vec<TheValue>, values: &Vec<TheValue>, _env: &TheExeEnvironment| {
                    stack.push(values[0].clone());
                };

                ctx.stack.push(value.clone());

                TheExeNode::new(call, vec![value.clone()])
            }
            TheAtom::Add() => {
                let call: TheExeNodeCall = |stack: &mut Vec<TheValue>, _values: &Vec<TheValue>, _env: &TheExeEnvironment| {
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
                let call: TheExeNodeCall = |stack: &mut Vec<TheValue>, _values: &Vec<TheValue>, _env: &TheExeEnvironment| {
                    let a = stack.pop().unwrap().to_i32().unwrap();
                    let b = stack.pop().unwrap().to_i32().unwrap();
                    stack.push(TheValue::Int(a * b));
                };

                TheExeNode::new(call, vec![])
            }
        }
    }

    pub fn to_kind(&self) -> TheAtomKind {
        match self {
            TheAtom::Stop => TheAtomKind::Eof,
            TheAtom::Variable(_name) => TheAtomKind::Identifier,
            TheAtom::Value(_value) => TheAtomKind::Number,
            TheAtom::Add() => TheAtomKind::Plus,
            TheAtom::Multiply() => TheAtomKind::Star,
        }
    }

    pub fn describe(&self) -> String {
        match self {
            TheAtom::Stop => "Stop".to_string(),
            TheAtom::Variable(name) => name.clone(), //"name".to_string(),
            TheAtom::Value(value) => value.describe(),
            TheAtom::Add() => "+".to_string(),
            TheAtom::Multiply() => "*".to_string(),
        }
    }

    #[cfg(feature = "ui")]
    /// Generates a text layout to edit the properties of the atom
    pub fn to_layout(&self, layout: &mut dyn TheHLayoutTrait) {
        match self {
            TheAtom::Variable(name) => {
                let mut text = TheText::new(TheId::empty());
                text.set_text("Variable Name:".to_string());
                let mut name_edit = TheTextLineEdit::new(TheId::named("Atom Variable"));
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
