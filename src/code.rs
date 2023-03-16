use anyhow::bail;
use num_enum::FromPrimitive;
use std::fmt;

use crate::{Result, Value};

type Bytecode = u16;

#[derive(
    Copy,
    Clone,
    Debug,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    FromPrimitive,
    num_enum::Default
)]
#[repr(u8)]
pub enum Op {
    Nil,
    True,
    False,
    Pop,
    Print,
    Return,
    Not,
    Negate,
    Equal,
    Greater,
    Less,
    Add,
    Subtract,
    Multiply,
    Divide,
    Constant,
    DefineGlobal,
    GetGlobal,
    SetGlobal,
    GetLocal,
    SetLocal,
    Extend,
    #[num_enum(default)]
    Unknown,
}

impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format!("OP_{self:?}").to_ascii_uppercase())
    }
}

#[derive(Copy, Clone)]
pub struct Instruction {
    opcode: Op,
    operand: u32,
    len: usize,
}

impl Instruction {
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn opcode(&self) -> Op {
        self.opcode
    }

    pub fn operand(&self) -> u32 {
        self.operand
    }
}

impl Default for Instruction {
    fn default() -> Self {
        Instruction {
            opcode: Op::default(),
            operand: 0,
            len: 1,
        }
    }
}

pub struct Chunk {
    code: Vec<Bytecode>,
    constants: Vec<Value>,
    line_map: LineMap,
}

impl Default for Chunk {
    fn default() -> Self {
        Chunk::new()
    }
}

impl Chunk {
    const MAX_CONSTS: usize = 0xffffff;

    pub(crate) fn new() -> Self {
        Chunk {
            code: Vec::new(),
            constants: Vec::new(),
            line_map: LineMap::new(),
        }
    }

    pub fn instructions(&self) -> InstIter {
        InstIter {
            chunk: self,
            offset: 0,
        }
    }

    fn get_instruction(&self, offset: usize) -> Instruction {
        assert!(offset < self.code.len());
        let mut inst = Instruction::default();
        let mut idx = offset;
        loop {
            let bytes = self.code[idx].to_be_bytes();
            inst.opcode = Op::from_primitive(bytes[0]);
            inst.operand |= bytes[1] as u32;
            if inst.opcode != Op::Extend {
                break;
            }
            idx += 1;
            inst.operand <<= 8;
            inst.len += 1;
        }
        inst
    }

    pub(crate) fn new_line(&mut self, line: u32) {
        self.line_map.new_line(line);
    }

    fn push_op(&mut self, op: Op, arg: u8) {
        let code = u16::from_be_bytes([op as u8, arg]);
        self.code.push(code);
        self.line_map.add_op();
    }

    pub(crate) fn write_op(&mut self, op: Op) {
        assert!(op < Op::Constant);
        self.push_op(op, 0);
    }

    pub(crate) fn write_op_arg(&mut self, op: Op, arg: u32) {
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

    pub(crate) fn add_constant(&mut self, value: Value) -> Result<u32> {
        let idx = self.constants.len();
        if idx > Chunk::MAX_CONSTS {
            bail!("too many constants in one chunk")
        }
        self.constants.push(value);
        Ok(idx as u32)
    }

    pub(crate) fn get_line(&self, offset: usize) -> u32 {
        self.line_map.get_line(offset)
    }

    pub(crate) fn get_constant(&self, idx: u32) -> Value {
        self.constants[idx as usize].clone()
    }
}

pub struct InstIter<'a> {
    chunk: &'a Chunk,
    pub(super) offset: usize,
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

// TODO: more efficient representation
struct LineMap {
    lines: Vec<u32>,
    current_line: u32,
}

impl Default for LineMap {
    fn default() -> Self {
        LineMap::new()
    }
}

impl LineMap {
    fn new() -> Self {
        LineMap {
            lines: Vec::new(),
            current_line: 1,
        }
    }

    fn new_line(&mut self, line: u32) {
        self.current_line = line;
    }

    fn add_op(&mut self) {
        self.lines.push(self.current_line);
    }

    fn get_line(&self, offset: usize) -> u32 {
        self.lines[offset]
    }
}

#[cfg(debug_assertions)]
impl Chunk {
    pub fn disassemble<T: fmt::Display>(&self, name: &str, sym_names: &[T]) {
        println!("== {name} ==");
        let mut offset = 0;
        for inst in self.instructions() {
            print!("{:4} ", self.get_line(offset));
            self.disassemble_instruction(inst, offset, sym_names);
            offset += inst.len;
        }
    }

    pub fn disassemble_instruction<T: fmt::Display>(
        &self,
        inst: Instruction,
        offset: usize,
        sym_names: &[T],
    ) {
        print!("{:04} ", offset);
        match inst.opcode {
            op if op < Op::Constant => {
                println!("{}", op);
            }
            Op::Unknown => println!("Unknown opcode {}", inst.opcode as u8),
            Op::Constant => {
                self.disassemble_const(inst.operand);
            }
            Op::DefineGlobal | Op::GetGlobal | Op::SetGlobal => {
                self.disassemble_sym(inst.opcode, inst.operand, sym_names);
            }
            _ => {
                Chunk::disassemble_op_arg(inst.opcode, inst.operand);
                println!();
            }
        }
    }

    fn disassemble_op_arg(op: Op, arg: u32) {
        print!("{:10} {:08} ", format!("{}", op), arg);
    }

    fn disassemble_const(&self, arg: u32) {
        Chunk::disassemble_op_arg(Op::Constant, arg);
        if arg as usize >= self.constants.len() {
            println!("(out of range)");
        } else {
            println!("{}", self.constants[arg as usize]);
        }
    }

    fn disassemble_sym<T: fmt::Display>(
        &self,
        op: Op,
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
}
