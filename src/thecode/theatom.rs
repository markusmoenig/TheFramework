use crate::prelude::*;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum TheAtom {
    Value(TheValue),
    Add(),
    Multiply(),
    Stop,
}

impl TheAtom {
    pub fn to_node(&self) -> TheExeNode {
        match self {
            TheAtom::Stop => {
                let call: TheExeNodeCall = |_stack: &mut Vec<TheValue>, _values: &Vec<TheValue>| {};
                TheExeNode::new(call, vec![])
            }
            TheAtom::Value(value) => {
                let call: TheExeNodeCall = |stack: &mut Vec<TheValue>, values: &Vec<TheValue>| {
                    stack.push(values[0].clone());
                };

                TheExeNode::new(call, vec![value.clone()])
            }
            TheAtom::Add() => {
                let call: TheExeNodeCall = |stack: &mut Vec<TheValue>, _values: &Vec<TheValue>| {
                    let a = stack.pop().unwrap().to_i32().unwrap();
                    let b = stack.pop().unwrap().to_i32().unwrap();
                    stack.push(TheValue::Int(a + b));
                };

                TheExeNode::new(call, vec![])
            }
            TheAtom::Multiply() => {
                let call: TheExeNodeCall = |stack: &mut Vec<TheValue>, _values: &Vec<TheValue>| {
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
            TheAtom::Value(_value) => TheAtomKind::Number,
            TheAtom::Add() => TheAtomKind::Plus,
            TheAtom::Multiply() => TheAtomKind::Star,
        }
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
