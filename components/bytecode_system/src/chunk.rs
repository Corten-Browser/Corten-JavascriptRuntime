//! Bytecode chunk - compiled bytecode container
//!
//! Contains instructions, constants, and metadata for execution.

use crate::instruction::{Instruction, SourcePosition};
use crate::opcode::{Opcode, RegisterId, UpvalueDescriptor};
use crate::optimizer::Optimizer;
use crate::value::Value;

/// A compiled bytecode chunk containing instructions and constants
#[derive(Debug, Clone, PartialEq)]
pub struct BytecodeChunk {
    /// Sequence of bytecode instructions
    pub instructions: Vec<Instruction>,
    /// Constant pool for literal values
    pub constants: Vec<Value>,
    /// Number of registers needed for execution
    pub register_count: u32,
    /// Nested function bytecode chunks (for closures)
    pub nested_functions: Vec<BytecodeChunk>,
}

impl BytecodeChunk {
    /// Create a new empty bytecode chunk
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            constants: Vec::new(),
            register_count: 0,
            nested_functions: Vec::new(),
        }
    }

    /// Get a reference to nested functions
    pub fn nested_functions(&self) -> &[BytecodeChunk] {
        &self.nested_functions
    }

    /// Add a nested function and return its index
    pub fn add_nested_function(&mut self, chunk: BytecodeChunk) -> usize {
        let idx = self.nested_functions.len();
        self.nested_functions.push(chunk);
        idx
    }

    /// Emit an instruction without source position
    pub fn emit(&mut self, opcode: Opcode) {
        self.instructions.push(Instruction::new(opcode));
    }

    /// Emit an instruction with source position
    pub fn emit_with_position(&mut self, opcode: Opcode, position: SourcePosition) {
        self.instructions
            .push(Instruction::with_position(opcode, position));
    }

    /// Add a constant to the constant pool and return its index
    pub fn add_constant(&mut self, value: Value) -> usize {
        let idx = self.constants.len();
        self.constants.push(value);
        idx
    }

    /// Run optimization passes on this chunk
    pub fn optimize(&mut self) {
        let mut optimizer = Optimizer::new();
        optimizer.optimize(self);
    }

    /// Get the number of instructions
    pub fn instruction_count(&self) -> usize {
        self.instructions.len()
    }

    /// Get the number of constants
    pub fn constant_count(&self) -> usize {
        self.constants.len()
    }

    /// Clear all instructions and constants
    pub fn clear(&mut self) {
        self.instructions.clear();
        self.constants.clear();
        self.register_count = 0;
    }

    /// Serialize chunk to binary format
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // Magic number
        bytes.extend_from_slice(b"BCNK");

        // Version
        bytes.push(1);

        // Register count
        bytes.extend_from_slice(&self.register_count.to_le_bytes());

        // Constants count and data
        bytes.extend_from_slice(&(self.constants.len() as u32).to_le_bytes());
        for constant in &self.constants {
            let const_bytes = constant.to_bytes();
            bytes.extend_from_slice(&const_bytes);
        }

        // Instructions count
        bytes.extend_from_slice(&(self.instructions.len() as u32).to_le_bytes());

        // Instructions
        for inst in &self.instructions {
            let inst_bytes = self.encode_instruction(inst);
            bytes.extend_from_slice(&inst_bytes);
        }

        bytes
    }

    /// Deserialize chunk from binary format
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 13 {
            return Err("Too few bytes for chunk header".to_string());
        }

        // Check magic number
        if &bytes[0..4] != b"BCNK" {
            return Err("Invalid magic number".to_string());
        }

        // Check version
        if bytes[4] != 1 {
            return Err(format!("Unsupported version: {}", bytes[4]));
        }

        let mut offset = 5;

        // Register count
        let register_count = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
        offset += 4;

        // Constants
        let const_count =
            u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;

        let mut constants = Vec::with_capacity(const_count);
        for _ in 0..const_count {
            let (value, consumed) = Value::from_bytes(&bytes[offset..])?;
            constants.push(value);
            offset += consumed;
        }

        // Instructions
        if offset + 4 > bytes.len() {
            return Err("Not enough bytes for instruction count".to_string());
        }
        let inst_count = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;

        let mut instructions = Vec::with_capacity(inst_count);
        for _ in 0..inst_count {
            let (inst, consumed) = Self::decode_instruction(&bytes[offset..])?;
            instructions.push(inst);
            offset += consumed;
        }

        Ok(Self {
            instructions,
            constants,
            register_count,
            nested_functions: Vec::new(), // TODO: Serialize nested functions
        })
    }

    /// Encode a single instruction to bytes
    fn encode_instruction(&self, inst: &Instruction) -> Vec<u8> {
        let mut bytes = Vec::new();

        // Encode opcode
        let (tag, data) = self.encode_opcode(&inst.opcode);
        bytes.push(tag);
        bytes.extend_from_slice(&data);

        // Encode source position
        match &inst.source_position {
            Some(pos) => {
                bytes.push(1); // Has position
                bytes.extend_from_slice(&pos.line.to_le_bytes());
                bytes.extend_from_slice(&pos.column.to_le_bytes());
                bytes.extend_from_slice(&pos.offset.to_le_bytes());
            }
            None => {
                bytes.push(0); // No position
            }
        }

        bytes
    }

    /// Encode opcode to tag and data bytes
    fn encode_opcode(&self, opcode: &Opcode) -> (u8, Vec<u8>) {
        match opcode {
            Opcode::LoadConstant(idx) => (0, (*idx as u32).to_le_bytes().to_vec()),
            Opcode::LoadUndefined => (1, vec![]),
            Opcode::LoadNull => (2, vec![]),
            Opcode::LoadTrue => (3, vec![]),
            Opcode::LoadFalse => (4, vec![]),
            Opcode::LoadGlobal(s) => {
                let s_bytes = s.as_bytes();
                let mut data = (s_bytes.len() as u32).to_le_bytes().to_vec();
                data.extend_from_slice(s_bytes);
                (5, data)
            }
            Opcode::StoreGlobal(s) => {
                let s_bytes = s.as_bytes();
                let mut data = (s_bytes.len() as u32).to_le_bytes().to_vec();
                data.extend_from_slice(s_bytes);
                (6, data)
            }
            Opcode::LoadLocal(reg) => (7, reg.0.to_le_bytes().to_vec()),
            Opcode::StoreLocal(reg) => (8, reg.0.to_le_bytes().to_vec()),
            Opcode::Add => (9, vec![]),
            Opcode::Sub => (10, vec![]),
            Opcode::Mul => (11, vec![]),
            Opcode::Div => (12, vec![]),
            Opcode::Mod => (13, vec![]),
            Opcode::Neg => (14, vec![]),
            Opcode::Not => (44, vec![]),
            Opcode::Equal => (15, vec![]),
            Opcode::StrictEqual => (16, vec![]),
            Opcode::NotEqual => (17, vec![]),
            Opcode::StrictNotEqual => (18, vec![]),
            Opcode::LessThan => (19, vec![]),
            Opcode::LessThanEqual => (20, vec![]),
            Opcode::GreaterThan => (21, vec![]),
            Opcode::GreaterThanEqual => (22, vec![]),
            Opcode::Jump(offset) => (23, (*offset as u32).to_le_bytes().to_vec()),
            Opcode::JumpIfTrue(offset) => (24, (*offset as u32).to_le_bytes().to_vec()),
            Opcode::JumpIfFalse(offset) => (25, (*offset as u32).to_le_bytes().to_vec()),
            Opcode::Return => (26, vec![]),
            Opcode::CreateObject => (27, vec![]),
            Opcode::LoadProperty(s) => {
                let s_bytes = s.as_bytes();
                let mut data = (s_bytes.len() as u32).to_le_bytes().to_vec();
                data.extend_from_slice(s_bytes);
                (28, data)
            }
            Opcode::StoreProperty(s) => {
                let s_bytes = s.as_bytes();
                let mut data = (s_bytes.len() as u32).to_le_bytes().to_vec();
                data.extend_from_slice(s_bytes);
                (29, data)
            }
            Opcode::CreateClosure(idx, upvalues) => {
                let mut data = (*idx as u32).to_le_bytes().to_vec();
                // Encode upvalue count
                data.extend_from_slice(&(upvalues.len() as u32).to_le_bytes());
                // Encode each upvalue descriptor
                for upvalue in upvalues {
                    data.push(if upvalue.is_local { 1 } else { 0 });
                    data.extend_from_slice(&upvalue.index.to_le_bytes());
                }
                (30, data)
            }
            Opcode::Call(argc) => (31, vec![*argc]),
            Opcode::LoadUpvalue(idx) => (32, idx.to_le_bytes().to_vec()),
            Opcode::StoreUpvalue(idx) => (33, idx.to_le_bytes().to_vec()),
            Opcode::CloseUpvalue => (34, vec![]),
            Opcode::Throw => (35, vec![]),
            Opcode::PushTry(offset) => (36, (*offset as u32).to_le_bytes().to_vec()),
            Opcode::PopTry => (37, vec![]),
            Opcode::PushFinally(offset) => (38, (*offset as u32).to_le_bytes().to_vec()),
            Opcode::PopFinally => (39, vec![]),
            Opcode::Pop => (40, vec![]),
            Opcode::Dup => (43, vec![]),
            Opcode::Await => (41, vec![]),
            Opcode::CreateAsyncFunction(idx, upvalues) => {
                let mut data = (*idx as u32).to_le_bytes().to_vec();
                // Encode upvalue count
                data.extend_from_slice(&(upvalues.len() as u32).to_le_bytes());
                // Encode each upvalue descriptor
                for upvalue in upvalues {
                    data.push(if upvalue.is_local { 1 } else { 0 });
                    data.extend_from_slice(&upvalue.index.to_le_bytes());
                }
                (42, data)
            }
        }
    }

    /// Decode instruction from bytes
    fn decode_instruction(bytes: &[u8]) -> Result<(Instruction, usize), String> {
        if bytes.is_empty() {
            return Err("Empty bytes for instruction".to_string());
        }

        let (opcode, mut offset) = Self::decode_opcode(bytes)?;

        // Decode source position
        if offset >= bytes.len() {
            return Err("Not enough bytes for source position flag".to_string());
        }

        let has_pos = bytes[offset] != 0;
        offset += 1;

        let source_position = if has_pos {
            if offset + 12 > bytes.len() {
                return Err("Not enough bytes for source position".to_string());
            }
            let line = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
            offset += 4;
            let column = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
            offset += 4;
            let pos_offset = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
            offset += 4;
            Some(SourcePosition {
                line,
                column,
                offset: pos_offset,
            })
        } else {
            None
        };

        Ok((
            Instruction {
                opcode,
                source_position,
            },
            offset,
        ))
    }

    /// Decode opcode from bytes
    fn decode_opcode(bytes: &[u8]) -> Result<(Opcode, usize), String> {
        if bytes.is_empty() {
            return Err("Empty bytes for opcode".to_string());
        }

        let tag = bytes[0];
        let mut offset = 1;

        let opcode = match tag {
            0 => {
                let idx =
                    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
                offset += 4;
                Opcode::LoadConstant(idx)
            }
            1 => Opcode::LoadUndefined,
            2 => Opcode::LoadNull,
            3 => Opcode::LoadTrue,
            4 => Opcode::LoadFalse,
            5 => {
                let len =
                    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
                offset += 4;
                let s = String::from_utf8(bytes[offset..offset + len].to_vec())
                    .map_err(|e| format!("Invalid UTF-8: {}", e))?;
                offset += len;
                Opcode::LoadGlobal(s)
            }
            6 => {
                let len =
                    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
                offset += 4;
                let s = String::from_utf8(bytes[offset..offset + len].to_vec())
                    .map_err(|e| format!("Invalid UTF-8: {}", e))?;
                offset += len;
                Opcode::StoreGlobal(s)
            }
            7 => {
                let reg = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
                offset += 4;
                Opcode::LoadLocal(RegisterId(reg))
            }
            8 => {
                let reg = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
                offset += 4;
                Opcode::StoreLocal(RegisterId(reg))
            }
            9 => Opcode::Add,
            10 => Opcode::Sub,
            11 => Opcode::Mul,
            12 => Opcode::Div,
            13 => Opcode::Mod,
            14 => Opcode::Neg,
            44 => Opcode::Not,
            15 => Opcode::Equal,
            16 => Opcode::StrictEqual,
            17 => Opcode::NotEqual,
            18 => Opcode::StrictNotEqual,
            19 => Opcode::LessThan,
            20 => Opcode::LessThanEqual,
            21 => Opcode::GreaterThan,
            22 => Opcode::GreaterThanEqual,
            23 => {
                let off =
                    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
                offset += 4;
                Opcode::Jump(off)
            }
            24 => {
                let off =
                    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
                offset += 4;
                Opcode::JumpIfTrue(off)
            }
            25 => {
                let off =
                    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
                offset += 4;
                Opcode::JumpIfFalse(off)
            }
            26 => Opcode::Return,
            27 => Opcode::CreateObject,
            28 => {
                let len =
                    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
                offset += 4;
                let s = String::from_utf8(bytes[offset..offset + len].to_vec())
                    .map_err(|e| format!("Invalid UTF-8: {}", e))?;
                offset += len;
                Opcode::LoadProperty(s)
            }
            29 => {
                let len =
                    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
                offset += 4;
                let s = String::from_utf8(bytes[offset..offset + len].to_vec())
                    .map_err(|e| format!("Invalid UTF-8: {}", e))?;
                offset += len;
                Opcode::StoreProperty(s)
            }
            30 => {
                let idx =
                    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
                offset += 4;
                let upvalue_count =
                    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
                offset += 4;
                let mut upvalues = Vec::with_capacity(upvalue_count);
                for _ in 0..upvalue_count {
                    let is_local = bytes[offset] != 0;
                    offset += 1;
                    let index =
                        u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
                    offset += 4;
                    upvalues.push(UpvalueDescriptor::new(is_local, index));
                }
                Opcode::CreateClosure(idx, upvalues)
            }
            31 => {
                let argc = bytes[offset];
                offset += 1;
                Opcode::Call(argc)
            }
            32 => {
                let idx = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
                offset += 4;
                Opcode::LoadUpvalue(idx)
            }
            33 => {
                let idx = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
                offset += 4;
                Opcode::StoreUpvalue(idx)
            }
            34 => Opcode::CloseUpvalue,
            35 => Opcode::Throw,
            36 => {
                let off =
                    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
                offset += 4;
                Opcode::PushTry(off)
            }
            37 => Opcode::PopTry,
            38 => {
                let off =
                    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
                offset += 4;
                Opcode::PushFinally(off)
            }
            39 => Opcode::PopFinally,
            40 => Opcode::Pop,
            41 => Opcode::Await,
            42 => {
                let idx =
                    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
                offset += 4;
                let upvalue_count =
                    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
                offset += 4;
                let mut upvalues = Vec::with_capacity(upvalue_count);
                for _ in 0..upvalue_count {
                    let is_local = bytes[offset] != 0;
                    offset += 1;
                    let index =
                        u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
                    offset += 4;
                    upvalues.push(UpvalueDescriptor::new(is_local, index));
                }
                Opcode::CreateAsyncFunction(idx, upvalues)
            }
            43 => Opcode::Dup,
            _ => return Err(format!("Unknown opcode tag: {}", tag)),
        };

        Ok((opcode, offset))
    }
}

impl Default for BytecodeChunk {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_default() {
        let chunk = BytecodeChunk::default();
        assert_eq!(chunk.instructions.len(), 0);
    }

    #[test]
    fn test_chunk_serialization_roundtrip() {
        let mut chunk = BytecodeChunk::new();
        chunk.add_constant(Value::Number(42.0));
        chunk.add_constant(Value::String("test".to_string()));
        chunk.emit(Opcode::LoadConstant(0));
        chunk.emit(Opcode::LoadConstant(1));
        chunk.emit(Opcode::Add);
        chunk.emit_with_position(
            Opcode::Return,
            SourcePosition {
                line: 10,
                column: 5,
                offset: 100,
            },
        );
        chunk.register_count = 5;

        let bytes = chunk.to_bytes();
        let restored = BytecodeChunk::from_bytes(&bytes).unwrap();

        assert_eq!(chunk.constants.len(), restored.constants.len());
        assert_eq!(chunk.instructions.len(), restored.instructions.len());
        assert_eq!(chunk.register_count, restored.register_count);
    }
}
