use crate::prelude::*;

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum TheInstruction {
    Constant(usize),
    Nil,
    False,
    True,
    Pop,
    GetLocal(usize),
    SetLocal(usize),
    GetGlobal(usize),
    DefineGlobal(usize),
    SetGlobal(usize),
    Equal,
    Greater,
    Less,
    Add,
    Subtract,
    Multiply,
    Divide,
    Not,
    Negate,
    Print,
    Jump(usize),
    JumpIfFalse(usize),
    Loop(usize),
    Call(usize),
    Return,
}

#[derive(PartialEq, Debug, Clone)]
pub struct TheChunk {
    pub code: Vec<TheInstruction>,
    pub lines: Vec<usize>,
    pub constants: Values,

    pub count: usize,
    pub capacity: usize,
}

impl Default for TheChunk {
    fn default() -> Self {
        Self::new()
    }
}

impl TheChunk {
    pub fn new() -> Self {
        Self {
            code: vec![],
            lines: vec![],
            constants: Values::new(),
            count: 0,
            capacity: 0,
        }
    }

    /// Clears the chunk.
    pub fn clear(&mut self) {
        self.code.clear();
        self.lines.clear();
        self.count = 0;
        self.capacity = 0;
    }

    /// Resizes the chunk to the new size
    pub fn resize(&mut self, new_size: usize) {
        if new_size == 0 {
            self.clear();
        }
        self.code.resize(new_size, TheInstruction::Return);
        self.lines.resize(new_size, 0);
        self.capacity = new_size;
    }

    /// Writes an instruction to current location
    pub fn write(&mut self, value: TheInstruction, line: usize) {
        if self.capacity < self.count + 1 {
            let capacity = if self.capacity < 8 {
                8
            } else {
                self.capacity * 2
            };
            self.resize(capacity);
            self.lines.resize(capacity, 0);
        }
        self.code[self.count] = value;
        self.lines[self.count] = line;
        self.count += 1;
    }

    /// Add a constant
    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.write(value);
        self.constants.count - 1
    }

    /// Debug the whole chunk
    pub fn disassemble(&self, name: &str) {
        println!("== {} ==", name);
        for offset in 0..self.count {
            self.disassemble_instruction(offset);
        }
    }

    /// Debug the instruction at the given offset
    pub fn disassemble_instruction(&self, offset: usize) {
        if let TheInstruction::Constant(index) = self.code[offset] {
            println!(
                "{} {} {:?} {:?}",
                offset, self.lines[offset], self.code[offset], self.constants.values[index]
            );
        } else {
            println!("{} {} {:?}", offset, self.lines[offset], self.code[offset]);
        }
    }
}
