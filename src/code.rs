use std::fmt::Display;

use anyhow::{bail, Result};

use crate::Value;

#[allow(non_upper_case_globals)]
#[allow(non_snake_case)]
pub(crate) mod Op {
    pub(super) fn name(op: u8) -> &'static str {
        match op {
            Nil => "NIL",
            True => "TRUE",
            False => "FALSE",
            Pop => "POP",
            Print => "PRINT",
            Return => "RETURN",
            Not => "NOT",
            Negate => "NEGATE",
            Equal => "EQUAL",
            Greater => "GREATER",
            Less => "LESS",
            Add => "ADD",
            Subtract => "SUBTRACT",
            Multiply => "MULTIPLY",
            Divide => "DIVIDE",
            Nop => "NOP",
            Constant => "CONSTANT",
            PopN => "POPN",
            DefineGlobal => "DEFINEGLOBAL",
            GetGlobal => "GETGLOBAL",
            SetGlobal => "SETGLOBAL",
            GetLocal => "GETLOCAL",
            SetLocal => "SETLOCAL",
            JumpIfFalse => "JUMPIFFALSE",
            Jump => "JUMP",
            Loop => "LOOP",
            Extend => "EXTEND",
            Call => "CALL",
            _ => "(unknown)",
        }
    }

    // Zero-argument opcodes
    pub const Nil: u8 = 0;
    pub const True: u8 = 1;
    pub const False: u8 = 2;
    pub const Pop: u8 = 3;
    pub const Print: u8 = 4;
    pub const Return: u8 = 5;
    pub const Not: u8 = 6;
    pub const Negate: u8 = 7;
    pub const Equal: u8 = 8;
    pub const Greater: u8 = 9;
    pub const Less: u8 = 10;
    pub const Add: u8 = 11;
    pub const Subtract: u8 = 12;
    pub const Multiply: u8 = 13;
    pub const Divide: u8 = 14;
    pub const Nop: u8 = 127;
    // One-argument opcodes
    pub const Constant: u8 = 128;
    pub const PopN: u8 = 129;
    pub const DefineGlobal: u8 = 130;
    pub const GetGlobal: u8 = 131;
    pub const SetGlobal: u8 = 132;
    pub const GetLocal: u8 = 133;
    pub const SetLocal: u8 = 134;
    pub const JumpIfFalse: u8 = 135;
    pub const Jump: u8 = 136;
    pub const Loop: u8 = 137;
    pub const Extend: u8 = 138;
    pub const Call: u8 = 139;
}

pub(crate) struct Chunk {
    code: Vec<Bytecode>,
    constants: Vec<Value>,
    line_map: LineMap,
}

pub(crate) struct InstIter<'a> {
    chunk: &'a Chunk,
    pub(crate) offset: usize,
}

#[derive(Copy, Clone)]
pub(crate) struct Instruction {
    opcode: Opcode,
    operand: u32,
    len: usize,
}

struct LineMap {
    lines: Vec<u32>,
    current: u32,
}

type Bytecode = u16;
pub(crate) type Opcode = u8;

impl Chunk {
    const MAX_CONSTS: usize = 0xffffff;

    fn new() -> Self {
        Chunk {
            code: Vec::new(),
            constants: Vec::new(),
            line_map: LineMap::new(),
        }
    }

    pub(crate) fn add_constant(&mut self, value: Value) -> Result<u32> {
        let idx = self.constants.len();
        if idx >= Chunk::MAX_CONSTS {
            bail!("too many constants in one chunk")
        }
        self.constants.push(value);
        Ok(idx as u32)
    }

    pub(crate) fn disassemble<T: Display>(&self, name: &str, sym_names: &[T]) {
        println!("== {name} ==");
        let mut offset = 0;
        for inst in self.instructions(offset) {
            print!("{:4} ", self.get_line(offset));
            self.disassemble_instruction(inst, offset, sym_names);
            offset += inst.len;
        }
    }

    fn disassemble_const(&self, arg: u32) {
        Chunk::disassemble_op_arg(Op::Constant, arg);
        if arg as usize >= self.constants.len() {
            println!("(out of range)");
        } else {
            println!("{}", self.constants[arg as usize]);
        }
    }

    pub(crate) fn disassemble_instruction<T: Display>(
        &self,
        inst: Instruction,
        offset: usize,
        sym_names: &[T],
    ) {
        print!("{:04} ", offset);
        match inst.opcode {
            op if op < Op::Constant => {
                println!("{}", Op::name(op));
            }
            Op::Constant => {
                // Show the value of the constant
                self.disassemble_const(inst.operand);
            }
            Op::DefineGlobal | Op::GetGlobal | Op::SetGlobal => {
                // Show the name of the symbol
                self.disassemble_sym(inst.opcode, inst.operand, sym_names);
            }
            Op::JumpIfFalse | Op::Jump => {
                // Convert the offset argument to an address
                Chunk::disassemble_op_arg(
                    inst.opcode,
                    (offset + 2 + inst.operand as usize) as u32,
                );
            }
            Op::Loop => {
                // Convert the offset argument to an address
                Chunk::disassemble_op_arg(
                    Op::Loop,
                    (offset + inst.len - inst.operand as usize) as u32,
                );
            }
            _ => {
                Chunk::disassemble_op_arg(inst.opcode, inst.operand);
                println!();
            }
        }
    }

    fn disassemble_op_arg(op: Opcode, arg: u32) {
        print!("{:10} {:08} ", Op::name(op), arg);
    }

    fn disassemble_sym<T: Display>(
        &self,
        op: Opcode,
        arg: u32,
        sym_names: &[T],
    ) {
        Chunk::disassemble_op_arg(op, arg);
        if arg as usize >= sym_names.len() {
            println!("(out of range)");
        } else {
            println!("{}", sym_names[arg as usize]);
        }
    }

    pub(crate) fn get_constant(&self, idx: u32) -> Value {
        self.constants[idx as usize].clone()
    }

    fn get_instruction(&self, offset: usize) -> Instruction {
        assert!(offset < self.code.len());
        let mut inst = Instruction::default();
        let mut idx = offset;
        loop {
            let bytes = self.code[idx].to_be_bytes();
            inst.opcode = bytes[0];
            inst.operand |= bytes[1] as u32;
            if inst.opcode != Op::Extend {
                break;
            }
            idx += 1;
            if idx >= self.code.len() {
                break;
            }
            inst.operand <<= 8;
            inst.len += 1;
        }
        inst
    }

    pub(crate) fn get_line(&self, offset: usize) -> u32 {
        self.line_map.get_line(offset)
    }

    pub(crate) fn instructions(&self, offset: usize) -> InstIter {
        InstIter {
            chunk: self,
            offset,
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.code.len()
    }

    pub(crate) fn new_line(&mut self, line: u32) {
        self.line_map.new_line(line);
    }

    pub(crate) fn patch_jump(&mut self, offset: usize, delta: u16) {
        let code = u16::from_be_bytes([
            (self.code[offset] >> 8) as u8,
            (delta >> 8) as u8,
        ]);
        self.code[offset] = code;
        let code = u16::from_be_bytes([
            (self.code[offset + 1] >> 8) as u8,
            (delta & 0xff) as u8,
        ]);
        self.code[offset + 1] = code;
    }

    fn push_op(&mut self, op: Opcode, arg: u8) {
        let code = u16::from_be_bytes([op, arg]);
        self.code.push(code);
        self.line_map.add_op();
    }

    pub(crate) fn write_jump(&mut self, op: Opcode) -> usize {
        let offset = self.code.len();
        self.write_op_arg(op, 0xfff);
        offset
    }

    pub(crate) fn write_op(&mut self, op: Opcode) {
        assert!(op < Op::Constant);
        self.push_op(op, 0);
    }

    pub(crate) fn write_op_arg(&mut self, op: Opcode, arg: u32) {
        assert!(op >= Op::Constant);
        if arg > 0xff {
            let ext_arg = arg >> 8;
            let start = 3 - (32 - (ext_arg.leading_zeros() as usize)) / 8;
            for byte in &ext_arg.to_be_bytes()[start..] {
                self.push_op(Op::Extend, *byte);
            }
        }
        self.push_op(op, arg as u8);
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Chunk::new()
    }
}

impl<'a> Iterator for InstIter<'a> {
    type Item = Instruction;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset >= self.chunk.code.len() {
            return None;
        }
        let inst = self.chunk.get_instruction(self.offset);
        self.offset += inst.len;
        Some(inst)
    }
}

impl Instruction {
    fn new() -> Self {
        Instruction {
            opcode: Op::Nop,
            operand: 0,
            len: 1,
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.len
    }

    pub(crate) fn opcode(&self) -> Opcode {
        self.opcode
    }

    pub(crate) fn operand(&self) -> u32 {
        self.operand
    }
}

impl Default for Instruction {
    fn default() -> Self {
        Instruction::new()
    }
}

impl LineMap {
    fn new() -> Self {
        LineMap {
            lines: Vec::new(),
            current: 1,
        }
    }

    fn add_op(&mut self) {
        self.lines.push(self.current);
    }

    fn get_line(&self, offset: usize) -> u32 {
        self.lines[offset]
    }

    fn new_line(&mut self, line: u32) {
        self.current = line;
    }
}

impl Default for LineMap {
    fn default() -> Self {
        LineMap::new()
    }
}
