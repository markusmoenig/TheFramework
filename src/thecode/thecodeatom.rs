use crate::prelude::*;

use super::thecodenode::TheCodeNodeData;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum TheCodeAtom {
    Assignment(String),
    Value(TheValue),
    Add,
    Multiply,
    LocalGet(String),
    LocalSet(String),
    ObjectGet(String, String),
    ObjectSet(String, String),
    FuncDef(String),
    FuncCall(String),
    ExternalCall(String, String),
    FuncArg(String),
    Pulse,
    Return,
    EndOfExpression,
    EndOfCode,
    Switch,
    CaseCondition,
    CaseBody,
}

impl TheCodeAtom {
    pub fn uneven_slot(&self) -> bool {
        matches!(self, TheCodeAtom::Assignment(_name))
            || matches!(self, TheCodeAtom::Add)
            || matches!(self, TheCodeAtom::Multiply)
    }

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
            // Generates a pulse gate.
            TheCodeAtom::Pulse => {
                let call: TheCodeNodeCall =
                    |_stack: &mut Vec<TheValue>,
                     data: &mut TheCodeNodeData,
                     sandbox: &mut TheCodeSandbox| {
                        let count = data.values[0].to_i32().unwrap();
                        let max_value = data.values[1].to_i32().unwrap();
                        if count < max_value {
                            data.values[0] = TheValue::Int(count + 1);
                            if sandbox.debug_mode {
                                sandbox.set_debug_value(
                                    data.location,
                                    TheValue::Text(format!("{} / {}", count + 1, max_value)),
                                );
                            }
                            TheCodeNodeCallResult::Break
                        } else {
                            if sandbox.debug_mode {
                                sandbox.set_debug_value(
                                    data.location,
                                    TheValue::Text(format!("{} / {}", count + 1, max_value)),
                                );
                            }
                            data.values[0] = TheValue::Int(0);
                            if let Some(v) = data.sub_functions[0].execute(sandbox).pop() {
                                data.values[1] = v;
                            }
                            TheCodeNodeCallResult::Continue
                        }
                    };

                let mut node = TheCodeNode::new(
                    call,
                    TheCodeNodeData::location_values(
                        ctx.node_location,
                        vec![TheValue::Int(0), TheValue::Int(4)],
                    ),
                );

                if let Some(mut function) = ctx.remove_function() {
                    let mut sandbox = TheCodeSandbox::new();
                    if let Some(v) = function.execute(&mut sandbox).pop() {
                        if let TheValue::Int(_) = v {
                            node.data.values[1] = v;
                        }
                    }
                    node.data.sub_functions.push(function);
                }

                node
            }
            TheCodeAtom::Assignment(_op) => {
                let call: TheCodeNodeCall =
                    |_stack: &mut Vec<TheValue>,
                     _data: &mut TheCodeNodeData,
                     _sandbox: &mut TheCodeSandbox| {
                        TheCodeNodeCallResult::Continue
                    };
                TheCodeNode::new(call, TheCodeNodeData::location(ctx.node_location))
            }
            TheCodeAtom::FuncDef(_name) => {
                let call: TheCodeNodeCall =
                    |_stack: &mut Vec<TheValue>,
                     _data: &mut TheCodeNodeData,
                     _sandbox: &mut TheCodeSandbox| {
                        TheCodeNodeCallResult::Continue
                    };
                TheCodeNode::new(call, TheCodeNodeData::location(ctx.node_location))
            }
            TheCodeAtom::FuncArg(_name) => {
                let call: TheCodeNodeCall =
                    |_stack: &mut Vec<TheValue>,
                     _data: &mut TheCodeNodeData,
                     _sandbox: &mut TheCodeSandbox| {
                        TheCodeNodeCallResult::Continue
                    };
                TheCodeNode::new(call, TheCodeNodeData::location(ctx.node_location))
            }
            TheCodeAtom::Return => {
                // This is only called if the function has a return value.
                let call: TheCodeNodeCall =
                    |stack: &mut Vec<TheValue>,
                     _data: &mut TheCodeNodeData,
                     sandbox: &mut TheCodeSandbox| {
                        sandbox.func_rc = stack.pop();
                        TheCodeNodeCallResult::Continue
                    };
                TheCodeNode::new(call, TheCodeNodeData::location(ctx.node_location))
            }
            TheCodeAtom::FuncCall(name) | TheCodeAtom::ExternalCall(name, _) => {
                let call: TheCodeNodeCall =
                    |stack: &mut Vec<TheValue>,
                     data: &mut TheCodeNodeData,
                     sandbox: &mut TheCodeSandbox| {
                        if let Some(id) = sandbox.module_stack.last() {
                            if let Some(mut function) = sandbox
                                .get_function_cloned(*id, &data.values[0].to_string().unwrap())
                            {
                                let mut clone = function.clone();

                                // Insert the arguments (if any) into the clone locals

                                let arguments = clone.arguments.clone();
                                for arg in &arguments {
                                    //}.iter().enumerate() {
                                    if let Some(arg_value) = stack.pop() {
                                        clone.set_local(arg.clone(), arg_value);
                                    }
                                }

                                sandbox.call_stack.push(clone);
                                function.execute(sandbox);
                                if let Some(rc_value) = &sandbox.func_rc {
                                    stack.push(rc_value.clone());
                                }
                                sandbox.call_stack.pop();
                            } else {
                                sandbox.call_global(
                                    data.location,
                                    stack,
                                    &data.values[0].to_string().unwrap(),
                                )
                            }
                        }
                        TheCodeNodeCallResult::Continue
                    };
                TheCodeNode::new(
                    call,
                    TheCodeNodeData::location_values(
                        ctx.node_location,
                        vec![TheValue::Text(name.clone())],
                    ),
                )
            }
            TheCodeAtom::LocalGet(name) => {
                let call: TheCodeNodeCall =
                    |stack: &mut Vec<TheValue>,
                     data: &mut TheCodeNodeData,
                     sandbox: &mut TheCodeSandbox| {
                        if let Some(function) = sandbox.call_stack.last_mut() {
                            if let Some(local) =
                                function.get_local(&data.values[0].to_string().unwrap())
                            {
                                stack.push(local.clone());
                            } else {
                                println!(
                                    "Runtime error: Unknown local variable {}.",
                                    &data.values[0].to_string().unwrap()
                                );
                            }
                        }
                        TheCodeNodeCallResult::Continue
                    };

                if ctx.error.is_none() {
                    let mut error = true;
                    if let Some(local) = ctx.local.last_mut() {
                        if let Some(local) = local.get(&name.clone()) {
                            ctx.stack.push(local.clone());
                            error = false;
                        }
                    }
                    if error {
                        ctx.error = Some(TheCompilerError::new(
                            ctx.node_location,
                            format!("Unknown local variable {}.", name),
                        ));
                    }
                }
                TheCodeNode::new(
                    call,
                    TheCodeNodeData::location_values(
                        ctx.node_location,
                        vec![TheValue::Text(name.clone())],
                    ),
                )
            }
            TheCodeAtom::LocalSet(name) => {
                let call: TheCodeNodeCall =
                    |stack: &mut Vec<TheValue>,
                     data: &mut TheCodeNodeData,
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

                        TheCodeNodeCallResult::Continue
                    };

                if ctx.error.is_none() {
                    if ctx.stack.is_empty() {
                        ctx.error = Some(TheCompilerError::new(
                            ctx.node_location,
                            "Nothing to assign to local variable.".to_string(),
                        ));
                    } else if let Some(local) = ctx.local.last_mut() {
                        local.set(name.clone(), ctx.stack.pop().unwrap());
                    }
                }

                TheCodeNode::new(
                    call,
                    TheCodeNodeData::location_values(
                        ctx.node_location,
                        vec![TheValue::Text(name.clone())],
                    ),
                )
            }
            TheCodeAtom::ObjectGet(object, name) => {
                let call: TheCodeNodeCall =
                    |stack: &mut Vec<TheValue>,
                     data: &mut TheCodeNodeData,
                     sandbox: &mut TheCodeSandbox| {
                        if let Some(object) =
                            sandbox.get_object(&data.values[0].to_string().unwrap())
                        {
                            if let Some(v) = object.get(&data.values[1].to_string().unwrap()) {
                                stack.push(v.clone());
                            } else {
                                println!(
                                    "Runtime error: Unknown object variable {}.",
                                    &data.values[1].to_string().unwrap()
                                );
                            }
                        } else {
                            println!(
                                "Runtime error: Unknown object {}.",
                                &data.values[0].to_string().unwrap()
                            );
                        }
                        TheCodeNodeCallResult::Continue
                    };

                ctx.stack.push(TheValue::Int(0));
                // if ctx.error.is_none() {
                //     let mut error = true;
                //     if let Some(local) = ctx.local.last_mut() {
                //         if let Some(local) = local.get(&name.clone()) {
                //             ctx.stack.push(local.clone());
                //             error = false;
                //         }
                //     }
                //     if error {
                //         ctx.error = Some(TheCompilerError::new(
                //             ctx.current_location,
                //             format!("Unknown local variable {}.", name),
                //         ));
                //     }
                // }
                TheCodeNode::new(
                    call,
                    TheCodeNodeData::location_values(
                        ctx.node_location,
                        vec![TheValue::Text(object.clone()), TheValue::Text(name.clone())],
                    ),
                )
            }
            TheCodeAtom::ObjectSet(object, name) => {
                let call: TheCodeNodeCall =
                    |stack: &mut Vec<TheValue>,
                     data: &mut TheCodeNodeData,
                     sandbox: &mut TheCodeSandbox| {
                        let mut debug_value: Option<TheValue> = None;

                        let debug_mode = sandbox.debug_mode;
                        if let Some(object) =
                            sandbox.get_object_mut(&data.values[0].to_string().unwrap())
                        {
                            if let Some(v) = stack.pop() {
                                if debug_mode {
                                    debug_value = Some(v.clone());
                                }
                                object.set(data.values[1].to_string().unwrap(), v);
                            } else {
                                println!("Runtime error: Object Set. Stack is empty.",);
                            }
                        } else {
                            println!(
                                "Runtime error: Object Set. Unknown object {}.",
                                data.values[0].to_string().unwrap()
                            );
                        }

                        if let Some(debug_value) = debug_value {
                            sandbox.set_debug_value(data.location, debug_value);
                        }
                        TheCodeNodeCallResult::Continue
                    };

                if ctx.error.is_none() {
                    if ctx.stack.is_empty() {
                        ctx.error = Some(TheCompilerError::new(
                            ctx.node_location,
                            "Nothing to assign to local variable.".to_string(),
                        ));
                    } else if let Some(local) = ctx.local.last_mut() {
                        local.set(name.clone(), ctx.stack.pop().unwrap());
                    }
                }

                TheCodeNode::new(
                    call,
                    TheCodeNodeData::location_values(
                        ctx.node_location,
                        vec![TheValue::Text(object.clone()), TheValue::Text(name.clone())],
                    ),
                )
            }
            TheCodeAtom::Value(value) => {
                let call: TheCodeNodeCall =
                    |stack: &mut Vec<TheValue>,
                     data: &mut TheCodeNodeData,
                     _sandbox: &mut TheCodeSandbox| {
                        stack.push(data.values[0].clone());
                        TheCodeNodeCallResult::Continue
                    };

                ctx.stack.push(value.clone());

                TheCodeNode::new(
                    call,
                    TheCodeNodeData::location_values(ctx.node_location, vec![value.clone()]),
                )
            }
            TheCodeAtom::Add => {
                let call: TheCodeNodeCall =
                    |stack: &mut Vec<TheValue>,
                     _data: &mut TheCodeNodeData,
                     _sandbox: &mut TheCodeSandbox| {
                        if let Some(b) = stack.pop() {
                            if let Some(a) = stack.pop() {
                                println!("t {:?} {:?}", a, b);
                                if let Some(result) = TheValue::add(&a, &b) {
                                    stack.push(result);
                                } else {
                                    println!("Runtime error: Add. Invalid types.");
                                }
                            }
                        }
                        TheCodeNodeCallResult::Continue
                    };

                if ctx.error.is_none() && ctx.stack.len() < 2 {
                    ctx.error = Some(TheCompilerError::new(
                        ctx.node_location,
                        format!("Invalid stack for Add ({})", ctx.stack.len()),
                    ));
                }

                TheCodeNode::new(call, TheCodeNodeData::location(ctx.node_location))
            }
            TheCodeAtom::Multiply => {
                let call: TheCodeNodeCall =
                    |stack: &mut Vec<TheValue>,
                     _data: &mut TheCodeNodeData,
                     _sandbox: &mut TheCodeSandbox| {
                        let a = stack.pop().unwrap().to_i32().unwrap();
                        let b = stack.pop().unwrap().to_i32().unwrap();
                        stack.push(TheValue::Int(a * b));
                        TheCodeNodeCallResult::Continue
                    };

                if ctx.error.is_none() && ctx.stack.len() < 2 {
                    ctx.error = Some(TheCompilerError::new(
                        ctx.node_location,
                        format!("Invalid stack for Multiply ({})", ctx.stack.len()),
                    ));
                }
                TheCodeNode::new(call, TheCodeNodeData::location(ctx.current_location))
            }
            TheCodeAtom::EndOfCode | TheCodeAtom::EndOfExpression => {
                let call: TheCodeNodeCall =
                    |_stack: &mut Vec<TheValue>,
                     _data: &mut TheCodeNodeData,
                     _sandbox: &mut TheCodeSandbox| {
                        TheCodeNodeCallResult::Continue
                    };
                TheCodeNode::new(call, TheCodeNodeData::location(ctx.current_location))
            }
            TheCodeAtom::Switch => {
                let call: TheCodeNodeCall =
                    |_stack: &mut Vec<TheValue>,
                     _data: &mut TheCodeNodeData,
                     _sandbox: &mut TheCodeSandbox| {
                        TheCodeNodeCallResult::Continue
                    };
                TheCodeNode::new(call, TheCodeNodeData::location(ctx.current_location))
            }
            TheCodeAtom::CaseCondition => {
                let call: TheCodeNodeCall =
                    |_stack: &mut Vec<TheValue>,
                     _data: &mut TheCodeNodeData,
                     _sandbox: &mut TheCodeSandbox| {
                        TheCodeNodeCallResult::Continue
                    };
                TheCodeNode::new(call, TheCodeNodeData::location(ctx.current_location))
            }
            TheCodeAtom::CaseBody => {
                let call: TheCodeNodeCall =
                    |_stack: &mut Vec<TheValue>,
                     _data: &mut TheCodeNodeData,
                     _sandbox: &mut TheCodeSandbox| {
                        TheCodeNodeCallResult::Continue
                    };
                TheCodeNode::new(call, TheCodeNodeData::location(ctx.current_location))
            }
        }
    }

    pub fn to_sdf(&self, dim: TheDim, zoom: f32) -> TheSDF {
        match self {
            Self::Value(_) => TheSDF::Hexagon(dim),
            Self::Add | &Self::Multiply => {
                TheSDF::RoundedRect(dim, (0.0, 0.0, 0.0, 0.0))
                //TheSDF::Rhombus(dim)
            }
            Self::ObjectSet(_, _) | Self::LocalSet(_) | Self::Pulse => {
                TheSDF::RoundedRect(dim, (0.0, 0.0, 10.0 * zoom, 10.0 * zoom))
            }
            Self::FuncCall(_) | Self::ExternalCall(_, _) => {
                TheSDF::RoundedRect(dim, (10.0 * zoom, 10.0 * zoom, 10.0 * zoom, 10.0 * zoom))
            }
            _ => TheSDF::RoundedRect(dim, (0.0, 0.0, 0.0, 0.0)),
        }
    }

    pub fn to_kind(&self) -> TheCodeAtomKind {
        match self {
            TheCodeAtom::Assignment(_op) => TheCodeAtomKind::Equal,
            TheCodeAtom::FuncDef(_name) => TheCodeAtomKind::Fn,
            TheCodeAtom::FuncArg(_name) => TheCodeAtomKind::Identifier,
            TheCodeAtom::FuncCall(_name) => TheCodeAtomKind::Identifier,
            TheCodeAtom::ExternalCall(_, _) => TheCodeAtomKind::Identifier,
            TheCodeAtom::Pulse => TheCodeAtomKind::Identifier,
            TheCodeAtom::Return => TheCodeAtomKind::Return,
            TheCodeAtom::LocalGet(_name) => TheCodeAtomKind::Identifier,
            TheCodeAtom::LocalSet(_name) => TheCodeAtomKind::Identifier,
            TheCodeAtom::ObjectGet(_object, _name) => TheCodeAtomKind::Identifier,
            TheCodeAtom::ObjectSet(_object, _name) => TheCodeAtomKind::Identifier,
            TheCodeAtom::Value(_value) => TheCodeAtomKind::Number,
            TheCodeAtom::Add => TheCodeAtomKind::Plus,
            TheCodeAtom::Multiply => TheCodeAtomKind::Star,
            TheCodeAtom::EndOfExpression => TheCodeAtomKind::Semicolon,
            TheCodeAtom::EndOfCode => TheCodeAtomKind::Eof,
            TheCodeAtom::Switch => TheCodeAtomKind::If,
            TheCodeAtom::CaseCondition => TheCodeAtomKind::If,
            TheCodeAtom::CaseBody => TheCodeAtomKind::If,
        }
    }

    pub fn describe(&self) -> String {
        match self {
            TheCodeAtom::Assignment(op) => op.clone(),
            TheCodeAtom::FuncDef(name) => name.clone(),
            TheCodeAtom::FuncArg(name) => name.clone(),
            TheCodeAtom::FuncCall(name) => name.clone(),
            TheCodeAtom::ExternalCall(name, _) => name.clone(),
            TheCodeAtom::Pulse => "Pulse".to_string(),
            TheCodeAtom::Return => "Return".to_string(),
            TheCodeAtom::LocalGet(name) => name.clone(),
            TheCodeAtom::LocalSet(name) => name.clone(),
            TheCodeAtom::ObjectGet(object, name) => format!("{}.{}", object, name),
            TheCodeAtom::ObjectSet(object, name) => format!("{}.{}", object, name),
            TheCodeAtom::Value(value) => match value {
                TheValue::Tile(name, _id) => name.clone(),
                _ => value.describe(),
            },
            TheCodeAtom::Add => "+".to_string(),
            TheCodeAtom::Multiply => "*".to_string(),
            TheCodeAtom::EndOfExpression => ";".to_string(),
            TheCodeAtom::EndOfCode => "Stop".to_string(),
            TheCodeAtom::Switch => "Switch".to_string(),
            TheCodeAtom::CaseCondition => "Case".to_string(),
            TheCodeAtom::CaseBody => ":".to_string(),
        }
    }

    pub fn help(&self) -> String {
        match self {
            TheCodeAtom::Assignment(name) => format!("Assignment ({}).", name),
            TheCodeAtom::FuncDef(name) => format!("Function definition ({}).", name),
            TheCodeAtom::FuncArg(name) => format!("Function argument ({}).", name),
            TheCodeAtom::FuncCall(name) => format!(
                "Function call ({}). Values below will be passed as arguments.",
                name
            ),
            TheCodeAtom::ExternalCall(_, description) => description.clone(),
            TheCodeAtom::Pulse => "A pulsing gate. Counts up to the gate value.".to_string(),
            TheCodeAtom::Return => "Return from a function. Optionally with a value.".to_string(),
            TheCodeAtom::LocalGet(name) => format!("Get the value of a local variable ({}).", name),
            TheCodeAtom::LocalSet(name) => format!("Set a value to a local variable ({}).", name),
            TheCodeAtom::ObjectGet(object, name) => {
                format!("Get the value of an object variable ({}.{}).", object, name)
            }
            TheCodeAtom::ObjectSet(object, name) => {
                format!("Set a value to an object variable ({}.{}).", object, name)
            }
            TheCodeAtom::Value(value) => match value {
                TheValue::Bool(_v) => format!("Boolean constant ({}).", self.describe()),
                TheValue::CodeObject(_v) => "An Object.".to_string(),
                TheValue::Int(v) => format!("Integer constant ({}).", v),
                TheValue::Float(_v) => format!("Float constant ({}).", value.describe()),
                TheValue::Text(v) => format!("Text constant ({}).", v),
                TheValue::Char(v) => format!("Char constant ({}).", v),
                TheValue::Int2(v) => format!("Int2 constant ({}).", v),
                TheValue::Float2(v) => format!("Float2 constant ({}).", v),
                TheValue::Int3(v) => format!("Int3 constant ({}).", v),
                TheValue::Float3(v) => format!("Float3 constant ({}).", v),
                TheValue::Int4(v) => format!("Int4 constant ({}).", v),
                TheValue::Float4(v) => format!("Float4 constant ({}).", v),
                TheValue::Position(v) => format!("Position ({}).", v),
                TheValue::Tile(name, _id) => format!("Tile ({}).", name),
                TheValue::KeyCode(_v) => "Key Code value.".to_string(),
                TheValue::RangeI32(_v) => "Range value.".to_string(),
                TheValue::RangeF32(_v) => "Range value.".to_string(),
                TheValue::ColorObject(_v) => "Color.".to_string(),
                TheValue::Empty => "Empty value.".to_string(),
            },
            TheCodeAtom::Add => "Operator ('+')".to_string(),
            TheCodeAtom::Multiply => "Operator ('*')".to_string(),
            TheCodeAtom::EndOfExpression => ";".to_string(),
            TheCodeAtom::EndOfCode => "Stop".to_string(),
            TheCodeAtom::Switch => "Switch statement.".to_string(),
            TheCodeAtom::CaseCondition => "Switch 'Case' statement.".to_string(),
            TheCodeAtom::CaseBody => "Switch 'Case' body.".to_string(),
        }
    }

    #[cfg(feature = "ui")]
    /// Generates a text layout to edit the properties of the atom
    pub fn to_layout(&self, layout: &mut dyn TheHLayoutTrait) {
        match self {
            TheCodeAtom::FuncDef(name) => {
                let mut text = TheText::new(TheId::empty());
                text.set_text("Function Name".to_string());
                let mut name_edit = TheTextLineEdit::new(TheId::named("Atom Func Def"));
                name_edit.set_text(name.clone());
                name_edit.set_needs_redraw(true);
                layout.add_widget(Box::new(text));
                layout.add_widget(Box::new(name_edit));
            }
            TheCodeAtom::FuncArg(name) => {
                let mut text = TheText::new(TheId::empty());
                text.set_text("Argument Name".to_string());
                let mut name_edit = TheTextLineEdit::new(TheId::named("Atom Func Arg"));
                name_edit.set_text(name.clone());
                name_edit.set_needs_redraw(true);
                layout.add_widget(Box::new(text));
                layout.add_widget(Box::new(name_edit));
            }
            TheCodeAtom::FuncCall(name) => {
                let mut text = TheText::new(TheId::empty());
                text.set_text("Function Name".to_string());
                let mut name_edit = TheTextLineEdit::new(TheId::named("Atom Func Call"));
                name_edit.set_text(name.clone());
                name_edit.set_needs_redraw(true);
                layout.add_widget(Box::new(text));
                layout.add_widget(Box::new(name_edit));
            }
            TheCodeAtom::LocalGet(name) => {
                let mut text = TheText::new(TheId::empty());
                text.set_text("Variable Name".to_string());
                let mut name_edit = TheTextLineEdit::new(TheId::named("Atom Local Get"));
                name_edit.set_text(name.clone());
                name_edit.set_needs_redraw(true);
                layout.add_widget(Box::new(text));
                layout.add_widget(Box::new(name_edit));
            }
            TheCodeAtom::LocalSet(name) => {
                let mut text = TheText::new(TheId::empty());
                text.set_text("Variable Name".to_string());
                let mut name_edit = TheTextLineEdit::new(TheId::named("Atom Local Set"));
                name_edit.set_text(name.clone());
                name_edit.set_needs_redraw(true);
                layout.add_widget(Box::new(text));
                layout.add_widget(Box::new(name_edit));
            }
            TheCodeAtom::ObjectGet(object, name) => {
                let mut text = TheText::new(TheId::empty());
                text.set_text("Object Name".to_string());
                let mut name_edit = TheTextLineEdit::new(TheId::named("Atom Object Get Object"));
                name_edit.set_text(object.clone());
                name_edit.set_needs_redraw(true);
                layout.add_widget(Box::new(text));
                layout.add_widget(Box::new(name_edit));

                let mut text = TheText::new(TheId::empty());
                text.set_text("Variable Name".to_string());
                let mut name_edit = TheTextLineEdit::new(TheId::named("Atom Object Get Variable"));
                name_edit.set_text(name.clone());
                name_edit.set_needs_redraw(true);
                layout.add_widget(Box::new(text));
                layout.add_widget(Box::new(name_edit));
            }
            TheCodeAtom::ObjectSet(object, name) => {
                let mut text = TheText::new(TheId::empty());
                text.set_text("Object Name".to_string());
                let mut name_edit = TheTextLineEdit::new(TheId::named("Atom Object Set Object"));
                name_edit.set_text(object.clone());
                name_edit.set_needs_redraw(true);
                layout.add_widget(Box::new(text));
                layout.add_widget(Box::new(name_edit));

                let mut text = TheText::new(TheId::empty());
                text.set_text("Variable Name".to_string());
                let mut name_edit = TheTextLineEdit::new(TheId::named("Atom Object Set Variable"));
                name_edit.set_text(name.clone());
                name_edit.set_needs_redraw(true);
                layout.add_widget(Box::new(text));
                layout.add_widget(Box::new(name_edit));
            }
            TheCodeAtom::Value(value) => match value {
                TheValue::Position(v) => {
                    create_float2_widgets(layout, TheId::named("Atom Position"), vec2f(v.x, v.y));
                }
                TheValue::Int(v) => {
                    let mut text = TheText::new(TheId::empty());
                    text.set_text(value.to_kind());
                    let mut name_edit = TheTextLineEdit::new(TheId::named(
                        format!("Atom {}", value.to_kind()).as_str(),
                    ));
                    name_edit.set_range(TheValue::RangeI32(core::ops::RangeInclusive::new(
                        std::i32::MIN,
                        std::i32::MAX,
                    )));
                    name_edit.set_text(v.to_string());
                    name_edit.set_needs_redraw(true);
                    layout.add_widget(Box::new(text));
                    layout.add_widget(Box::new(name_edit));
                }
                _ => {
                    let mut text = TheText::new(TheId::empty());
                    text.set_text(value.to_kind());
                    let mut name_edit = TheTextLineEdit::new(TheId::named(
                        format!("Atom {}", value.to_kind()).as_str(),
                    ));
                    name_edit.set_text(value.describe());
                    name_edit.set_needs_redraw(true);
                    layout.add_widget(Box::new(text));
                    layout.add_widget(Box::new(name_edit));
                }
            },
            _ => {}
        };
    }

    // #[cfg(feature = "ui")]
    // / Generates a text layout to edit the properties of the atom
    // pub fn process_value_change(&mut self, name: String, value: TheValue) {
    //     match self {
    //         TheCodeAtom::Value(_) => {
    //             //println!("{} {:?}", name, value);
    //             if name == "Atom Integer Edit" {
    //                 *self = TheCodeAtom::Value(value.clone());
    //             }
    //         }
    //         _ => {}
    //     };
    // }
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
