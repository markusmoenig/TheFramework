// use std::mem::size_of_val;
use crate::prelude::*;
use TheInstruction::*;

#[derive(Clone, Debug, PartialEq)]
pub enum InterpretError {
    Ok,
    CompileError(String, usize),
    RuntimeError(String, usize),
}

const FRAMES_MAX: usize = 64;

pub struct CallFrame {
    function: ObjectFunction,
    ip: usize,
    slots: usize,
}

pub struct TheVM {
    frames: [CallFrame; FRAMES_MAX],
    frame_count: usize,
    frame_current: usize,

    compiler: TheCompiler,

    stack: Vec<Value>,

    globals: FxHashMap<String, Value>,
}

impl Default for TheVM {
    fn default() -> Self {
        Self::new()
    }
}

impl TheVM {
    pub fn new() -> Self {
        //let array = [(); STACK_MAX].map(|_| Value::Nil);

        //let val = Value::Float(2.2);
        //println!("{}", size_of_val(&val));

        Self {
            frames: [(); FRAMES_MAX].map(|_| CallFrame {
                function: ObjectFunction::new("".to_string(), FunctionType::Script),
                ip: 0,
                slots: 0,
            }),
            frame_count: 0,
            frame_current: 0,

            compiler: TheCompiler::new(),

            stack: vec![],
            //stack_top           : 0,
            globals: FxHashMap::default(),
        }
    }

    pub fn interpret(&mut self, code: String) -> Result<(), InterpretError> {
        let rc = self.compiler.compile(code);

        if let Ok(function) = rc.clone() {
            println!("{:?}", function.chunk.count);

            _ = self.call(function, 0);
            self.globals.clear();
            self.add_native("clock", NativeFunction(TheVM::clock));
            self.frame_count = 1;
            self.frame_current = 0;

            if self.frames[self.frame_current].function.chunk.count > 0 {
                let rc = self.run();
                println!("globals {:?}", self.globals);

                // let nodes = self.compiler.nodes.clone();
                // for n in &nodes {
                //     println!("running {}", n.name);

                //     self.frames[0].function.chunk = n.chunk.clone();
                //     self.frames[0].ip = 0;
                //     self.frames[0].slots = 0;
                //     self.stack = vec![];

                //     _ = self.run();
                // }
                rc
            } else {
                Ok(())
            }
        } else {
            Err(rc.err().unwrap())
        }
    }

    /// Adds a native function
    pub fn add_native(&mut self, name: &str, f: NativeFunction) {
        self.globals
            .insert(name.to_string(), Value::NativeFunction(f));
    }

    pub fn run(&mut self) -> Result<(), InterpretError> {
        self.reset_stack();

        loop {
            //let chunk = &self.chunk;
            let ip = self.frames[self.frame_current].ip;
            self.frames[self.frame_current].ip += 1;

            // Start Debug
            // let mut stack_str = "[".to_owned();
            // for i in 0..self.stack_top {
            //     stack_str += format!("{:?}", self.stack[i]).as_str();
            // }
            // stack_str += "]";
            // println!("{}", stack_str);
            //self.get_frame().function.chunk.disassemble_instruction(ip);
            // End Debug

            match self.frames[self.frame_current].function.chunk.code[ip] {
                Constant(index) => {
                    self.push(self.get_frame().function.chunk.constants.values[index].clone());
                }
                Nil => self.push(Value::Nil),
                True => self.push(Value::Bool(true)),
                False => self.push(Value::Bool(false)),
                Pop => _ = self.pop(),
                GetLocal(index) => {
                    let i = index + self.get_frame().slots;
                    let value = self.stack[i].clone();
                    self.push(value);
                }
                SetLocal(index) => {
                    let i = index + self.get_frame().slots;
                    self.stack[i] = self.peek(0);
                }
                GetGlobal(index) => {
                    let name = self.get_frame().function.chunk.constants.values[index].as_string();
                    if let Some(name) = name {
                        let value = self.globals.get(&name).cloned();
                        if let Some(value) = value {
                            self.push(value);
                        } else {
                            return Err(self.runtime_error(
                                format!("Undefined variable '{}'.", name).as_str(),
                            ));
                        }
                    }
                }
                DefineGlobal(index) => {
                    if let Some(name) =
                        self.get_frame().function.chunk.constants.values[index].as_string()
                    {
                        let value = self.pop();
                        self.globals.insert(name, value);
                    }
                }
                SetGlobal(index) => {
                    if let Some(name) =
                        self.get_frame().function.chunk.constants.values[index].as_string()
                    {
                        let value = self.pop();
                        if self.globals.contains_key(&name) {
                            self.globals.insert(name, value);
                        } else {
                            return Err(self.runtime_error(
                                format!("Undefined variable '{}'.", name).as_str(),
                            ));
                        }
                    }
                }
                Equal => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(Value::Bool(b.equal(&a)));
                }
                Greater => {
                    let b = self.pop();
                    let a = self.pop();
                    match a {
                        Value::Float(value_a) => match b {
                            Value::Float(value_b) => self.push(Value::Bool(value_a > value_b)),
                            _ => {}
                        },
                        _ => {}
                    }
                }
                Less => {
                    let b = self.pop();
                    let a = self.pop();
                    match a {
                        Value::Float(value_a) => match b {
                            Value::Float(value_b) => self.push(Value::Bool(value_a < value_b)),
                            _ => {}
                        },
                        _ => {}
                    }
                }
                Add => {
                    let b = self.pop();
                    let a = self.pop();
                    // println!("b {:?}", b);
                    // println!("a {:?}", a);
                    match a {
                        Value::Float(value_a) => match b {
                            Value::Float(value_b) => self.push(Value::Float(value_a + value_b)),
                            _ => {}
                        },
                        Value::String(value_a) => match b {
                            Value::String(value_b) => {
                                self.push(Value::String(value_a + value_b.as_str()))
                            }
                            _ => {}
                        },
                        _ => {
                            return Err(self.runtime_error("Operands must be numbers."));
                        }
                    }
                }
                Subtract => {
                    let b = self.pop();
                    let a = self.pop();
                    match a {
                        Value::Float(value_a) => match b {
                            Value::Float(value_b) => self.push(Value::Float(value_a - value_b)),
                            _ => {}
                        },
                        _ => {
                            return Err(self.runtime_error("Operands must be numbers."));
                        }
                    }
                }
                Multiply => {
                    let b = self.pop();
                    let a = self.pop();
                    match a {
                        Value::Float(value_a) => match b {
                            Value::Float(value_b) => self.push(Value::Float(value_a * value_b)),
                            _ => {}
                        },
                        _ => {
                            return Err(self.runtime_error("Operands must be numbers."));
                        }
                    }
                }
                Divide => {
                    let b = self.pop();
                    let a = self.pop();
                    match a {
                        Value::Float(value_a) => match b {
                            Value::Float(value_b) => self.push(Value::Float(value_a / value_b)),
                            _ => {}
                        },
                        _ => {
                            return Err(self.runtime_error("Operands must be numbers."));
                        }
                    }
                }
                Not => {
                    let v = self.pop();
                    let is_false = self.is_falsey(v);
                    self.push(Value::Bool(is_false));
                }
                Negate => {
                    let v = self.pop();
                    match v {
                        Value::Float(value) => self.push(Value::Float(-value)),
                        _ => {
                            return Err(self.runtime_error("Operand must be a number."));
                        }
                    }
                }
                Print => {
                    let value = self.pop();
                    value.print();
                }
                Jump(offset) => {
                    self.get_mut_frame().ip += offset;
                }
                JumpIfFalse(offset) => {
                    if self.is_falsey(self.peek(0)) {
                        self.get_mut_frame().ip += offset;
                    }
                }
                Loop(offset) => {
                    self.get_mut_frame().ip -= offset;
                }
                Call(arg_count) => {
                    let rc = self.call_value(self.peek(arg_count), arg_count);
                    if rc.is_err() {
                        return rc;
                    }
                }
                Return => {
                    /*
                    let value = self.pop();
                    self.frame_count -= 1;

                    if self.frame_count == 0 {
                        self.pop();
                        return Ok(());
                    } else {
                        self.frame_current -= 1;
                    }*/

                    //self.stack.truncate(self.frames[self.frame_count].slots);
                    //self.push(value);

                    return Ok(());
                }
            }
        }
    }

    /// Returns the current frame
    fn get_frame(&self) -> &CallFrame {
        &self.frames[self.frame_current]
    }

    /// Returns the current frame
    fn get_mut_frame(&mut self) -> &mut CallFrame {
        &mut self.frames[self.frame_current]
    }

    /// Calls a function
    fn call_value(&mut self, callee: Value, arg_count: usize) -> Result<(), InterpretError> {
        match callee {
            Value::Function(function) => {
                return self.call(function, arg_count);
            }
            Value::NativeFunction(function) => {
                return self.native_call(function, arg_count);
            }
            _ => return Err(self.runtime_error("Cannot call value.")),
        }
        //false
    }

    fn call(&mut self, function: ObjectFunction, arg_count: usize) -> Result<(), InterpretError> {
        if arg_count != function.arity {
            return Err(self.runtime_error("Wrong number of function arguments in call."));
        }
        self.frames[self.frame_count].ip = 0;
        self.frames[self.frame_count].slots = self.stack.len() - arg_count;
        self.frames[self.frame_count].function = function;
        self.frame_count += 1;
        self.frame_current += 1;
        Ok(())
    }

    fn native_call(
        &mut self,
        function: NativeFunction,
        arg_count: usize,
    ) -> Result<(), InterpretError> {
        let left = self.stack.len() - arg_count;
        let value = function.0(&self, &self.stack[left..]);
        self.stack.truncate(left - 1);
        self.push(value);
        Ok(())
    }

    /// Create a runtime error
    fn runtime_error(&mut self, message: &str) -> InterpretError {
        self.reset_stack();

        InterpretError::RuntimeError(
            message.to_string(),
            self.frames[self.frame_current].function.chunk.lines
                [self.frames[self.frame_current].function.chunk.count - 1],
        )
    }

    /// Push a value onto the stack
    pub fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    /// Pops a value of the stack
    pub fn pop(&mut self) -> Value {
        if self.stack.is_empty() == false {
            return self.stack.pop().expect("Stack is empty!");
        }
        Value::Nil
    }

    /// Returns true if the value is false
    fn is_falsey(&self, value: Value) -> bool {
        value == Value::Bool(false) || value == Value::Float(0.0) || value == Value::Nil
    }

    /// Peeks into the stack
    fn peek(&self, n: usize) -> Value {
        self.stack[self.stack.len() - 1 - n].clone()
    }

    fn clock(_vm: &TheVM, args: &[Value]) -> Value {
        println!("{:?}", args);
        Value::Float(10.0)
    }

    /// Reset the stack
    pub fn reset_stack(&mut self) {
        self.stack = vec![];
    }
}
