use crate::prelude::*;

use super::thecodenode::TheCodeNodeData;

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
            TheCodeAtom::FuncDef(_name) => {
                let call: TheCodeNodeCall =
                    |_stack: &mut Vec<TheValue>,
                     data: &TheCodeNodeData,
                     _sandbox: &mut TheCodeSandbox| {};
                TheCodeNode::new(call, TheCodeNodeData::location(ctx.current_location))
            }
            TheCodeAtom::Return => {
                // This is only called if the function has a return value.
                let call: TheCodeNodeCall =
                    |stack: &mut Vec<TheValue>,
                     _data: &TheCodeNodeData,
                     sandbox: &mut TheCodeSandbox| {
                        println!("inside return {:?}", stack);
                        sandbox.func_rc = stack.pop();
                    };
                TheCodeNode::new(call, TheCodeNodeData::location(ctx.current_location))
            }
            TheCodeAtom::FuncCall(name) => {
                let call: TheCodeNodeCall =
                    |stack: &mut Vec<TheValue>,
                     data: &TheCodeNodeData,
                     sandbox: &mut TheCodeSandbox| {
                        if let Some(id) = sandbox.module_stack.last() {
                            if let Some(mut function) = sandbox
                                .get_function_cloned(*id, &data.values[0].to_string().unwrap())
                            {
                                let clone = function.clone();
                                sandbox.call_stack.push(clone);
                                function.execute(sandbox);
                                println!("clone: {:?}", sandbox.call_stack.last());
                                println!("func_rc {:?}", sandbox.func_rc.clone());
                                if let Some(rc_value) = &sandbox.func_rc {
                                    stack.push(rc_value.clone());
                                }
                                sandbox.call_stack.pop();
                            }
                        }
                    };
                TheCodeNode::new(
                    call,
                    TheCodeNodeData::location_values(
                        ctx.current_location,
                        vec![TheValue::Text(name.clone())],
                    ),
                )
            }
            TheCodeAtom::LocalGet(name) => {
                let call: TheCodeNodeCall =
                    |stack: &mut Vec<TheValue>,
                     data: &TheCodeNodeData,
                     sandbox: &mut TheCodeSandbox| {
                        if let Some(function) = sandbox.call_stack.last_mut() {
                            if let Some(local) =
                                function.get_local(&data.values[0].to_string().unwrap())
                            {
                                stack.push(local.clone());
                            }
                        }
                    };

                let mut error = true;
                if let Some(local) = ctx.local.last_mut() {
                    if let Some(local) = local.get(&name.clone()) {
                        ctx.stack.push(local.clone());
                        error = false;
                    }
                }
                if error {
                    ctx.error = Some(TheCompilerError::new(
                        ctx.current_location,
                        format!("Unknown local variable {}.", name),
                    ));
                }
                TheCodeNode::new(
                    call,
                    TheCodeNodeData::location_values(
                        ctx.current_location,
                        vec![TheValue::Text(name.clone())],
                    ),
                )
            }
            TheCodeAtom::LocalSet(name) => {
                let call: TheCodeNodeCall =
                    |stack: &mut Vec<TheValue>,
                     data: &TheCodeNodeData,
                     sandbox: &mut TheCodeSandbox| {

                        let mut debug_value: Option<TheValue> = None;

                        if let Some(function) = sandbox.call_stack.last_mut() {
                            if let Some(local) = function.local.last_mut() {
                                let v = stack.pop().unwrap();
                                if sandbox.debug_mode {
                                    debug_value = Some(v.clone());
                                }
                                local.set(data.values[0].to_string().unwrap(), v);
                            }
                        }

                        if let Some(debug_value) = debug_value {
                            sandbox.set_debug_value(data.location, debug_value);
                        }
                    };

                if ctx.stack.is_empty() {
                    ctx.error = Some(TheCompilerError::new(
                        ctx.current_location,
                        "Nothing to assign to local variable.".to_string(),
                    ));
                } else if let Some(local) = ctx.local.last_mut() {
                    local.set(name.clone(), ctx.stack.pop().unwrap());
                }

                TheCodeNode::new(
                    call,
                    TheCodeNodeData::location_values(
                        ctx.node_location,
                        vec![TheValue::Text(name.clone())],
                    ),
                )
            }
            TheCodeAtom::Value(value) => {
                let call: TheCodeNodeCall =
                    |stack: &mut Vec<TheValue>,
                     data: &TheCodeNodeData,
                     _sandbox: &mut TheCodeSandbox| {
                        stack.push(data.values[0].clone());
                    };

                ctx.stack.push(value.clone());

                TheCodeNode::new(
                    call,
                    TheCodeNodeData::location_values(ctx.current_location, vec![value.clone()]),
                )
            }
            TheCodeAtom::Add => {
                let call: TheCodeNodeCall =
                    |stack: &mut Vec<TheValue>,
                     _data: &TheCodeNodeData,
                     _sandbox: &mut TheCodeSandbox| {
                        let a = stack.pop().unwrap().to_i32().unwrap();
                        let b = stack.pop().unwrap().to_i32().unwrap();
                        stack.push(TheValue::Int(a + b));
                    };

                if ctx.stack.len() < 2 {
                    ctx.error = Some(TheCompilerError::new(
                        ctx.current_location,
                        format!("Invalid stack for Add ({})", ctx.stack.len()),
                    ));
                }

                TheCodeNode::new(call, TheCodeNodeData::location(ctx.current_location))
            }
            TheCodeAtom::Multiply => {
                let call: TheCodeNodeCall =
                    |stack: &mut Vec<TheValue>,
                     _data: &TheCodeNodeData,
                     _sandbox: &mut TheCodeSandbox| {
                        let a = stack.pop().unwrap().to_i32().unwrap();
                        let b = stack.pop().unwrap().to_i32().unwrap();
                        stack.push(TheValue::Int(a * b));
                    };

                if ctx.stack.len() < 2 {
                    ctx.error = Some(TheCompilerError::new(
                        ctx.current_location,
                        format!("Invalid stack for Multiply ({})", ctx.stack.len()),
                    ));
                }
                TheCodeNode::new(call, TheCodeNodeData::location(ctx.current_location))
            }
            TheCodeAtom::EndOfCode | TheCodeAtom::EndOfExpression => {
                let call: TheCodeNodeCall =
                    |_stack: &mut Vec<TheValue>,
                     _data: &TheCodeNodeData,
                     _sandbox: &mut TheCodeSandbox| {};
                TheCodeNode::new(call, TheCodeNodeData::location(ctx.current_location))
            }
        }
    }

    pub fn to_kind(&self) -> TheCodeAtomKind {
        match self {
            TheCodeAtom::FuncDef(_name) => TheCodeAtomKind::Fn,
            TheCodeAtom::FuncCall(_name) => TheCodeAtomKind::Identifier,
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
