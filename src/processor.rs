/// Represents the function signature that an instruction operation's function
/// must adhere to.
type InstructionOperation = fn(p: &mut Processor, word: u32, address: u64) -> u64;

/// Represents one of the instructions of the RISC-V machine.
pub struct Instruction {
    /// A mask that can be AND'ed to the instruction word in order to retrieve the fields
    /// that matter for decoding.
    pub mask: u32,

    /// The result that AND'ing the word and the mask should produce for a positive match
    /// for this instruction.
    pub result: u32,

    /// The function that will emulate this instruction's operation.
    pub operation: InstructionOperation,

    /// Name of the instruction.
    pub name: &'static str,
}

/// Size in bytes of RISC-V instructions
/// This is a lie, as RISC-V allows for variable-length instructions from
/// 16 bytes to 64 bytes... but mostly only 32 bytes is used so :3
const INSTRUCTION_SZ: u64  = 4;

/// Masks an instruction's opcode field.
const MASK_OP: u32         = 0x7f;

/// Masks R-type instruction's opcode, funct3 and funct7 fields.
const MASK_OP_FN3_FN7: u32 = 0xfe00707f;

pub struct Processor {
    pub registers:  [u64; 33],
}

impl Processor {
    /// Size of the static `INSTRUCTIONS` array.
    const NUM_OF_INSTRUCTIONS: usize = 2;

    /// Static list of RISC-V implemented instructions
    const INSTRUCTIONS: [Instruction; Self::NUM_OF_INSTRUCTIONS] = [
	Instruction {
	    mask:      MASK_OP_FN3_FN7,
	    result:    0x33,
	    operation: op_add,
	    name:      "ADD"
	},
	Instruction {
	    mask:      MASK_OP,
	    result:    0x6f,
	    operation: op_jal,
	    name:      "JAL"
	},
    ];

    pub fn new(entrypoint: u64) -> Self {
	let mut processor = Processor {
            registers: [0u64; 33],
	};
	processor.set_reg(Register::Pc, entrypoint);
	processor
    }

    pub fn reg(&self, reg: Register) -> u64 {
        self.registers[reg as usize]
    }

    pub fn set_reg(&mut self, reg: Register, val: u64) {
        self.registers[reg as usize] = val;
    }

    pub fn inc_pc(&mut self) {
	self.set_reg(Register::Pc, self.reg(Register::Pc) + INSTRUCTION_SZ);
    }

    pub fn decode(word: u32) -> Option<&'static Instruction> {
	for i in 0..Self::NUM_OF_INSTRUCTIONS {
	    let ix = &Self::INSTRUCTIONS[i];

	    if (word & ix.mask) == ix.result {
		return Some(ix);
	    }
	}

	return None;
    }
}

pub fn op_add(_p: &mut Processor, _word: u32, _address: u64) -> u64 {
    unimplemented!();
}

/// JAL
/// Stores the address following the jump (pc + 4) into rd, then adds the
/// sign-extended immediate to pc.
pub fn op_jal(p: &mut Processor, word: u32, address: u64) -> u64 {
    let instr = InstrJ::from(word);
    let dest  = address.wrapping_add(instr.imm as i32 as i64 as u64);

    p.set_reg(instr.rd, address + INSTRUCTION_SZ);
    p.set_reg(Register::Pc, dest);

    dest
}

#[repr(usize)]
#[derive(Debug, Copy, Clone)]
pub enum Register {
    Zero = 0,
    Ra,
    Sp,
    Gp,
    Tp,
    T0,
    T1,
    T2,
    S0,
    S1,
    A0,
    A1,
    A2,
    A3,
    A4,
    A5,
    A6,
    A7,
    S2,
    S3,
    S4,
    S5,
    S6,
    S7,
    S8,
    S9,
    S10,
    S11,
    T3,
    T4,
    T5,
    T6,
    Pc
}

impl From<u32> for Register {
    fn from(v: u32) -> Self {
        assert!(v <= 33);

        unsafe {
            core::ptr::read_unaligned(&(v as usize) as *const usize as *const Register)
        }
    }
}

///  31                    25  24    20  19    15  14       12  11    7  6         0
///  -------------------------------------------------------------------------------
/// |         funct7         |   rs2   |   rs1   |   funct3   |   rd   |   opcode   |
///  -------------------------------------------------------------------------------
pub struct InstrR {
    pub funct7: u32,
    pub rs2:    Register,
    pub rs1:    Register,
    pub funct3: u32,
    pub rd:     Register,
}

impl From<u32> for InstrR {
    fn from(instr: u32) -> Self {
        Self {
            funct7: (instr >> 25) & 0b0111_1111,
            rs2: Register::from((instr >> 20) & 0b0001_1111),
            rs1: Register::from((instr >> 15) & 0b0001_1111),
            funct3: (instr >> 12) & 0b0111,
            rd: Register::from((instr >> 7) & 0b0001_1111),
        }
    }
}

///  31                              20  19    15  14       12  11    7  6         0
///  -------------------------------------------------------------------------------
/// |              imm110              |   rs1   |   funct3   |   rd   |   opcode   |
///  -------------------------------------------------------------------------------
pub struct InstrI {
    pub imm:    i32,
    pub rs1:    Register,
    pub funct3: u32,
    pub rd:     Register,
}

impl From<u32> for InstrI {
    fn from(instr: u32) -> Self {
        Self {
            imm: (instr as i32) >> 20,
            rs1: Register::from((instr >> 15) & 0b0001_1111),
            funct3: (instr >> 12) & 0b0111,
            rd: Register::from((instr >> 7) & 0b0001_1111),
        }
    }
}

///  31                    25  24    20  19    15  14       12  11    7  6         0
///  -------------------------------------------------------------------------------
/// |         imm115         |   rs2   |   rs1   |   funct3   |  imm4  |   opcode   |
///  -------------------------------------------------------------------------------
pub struct InstrS {
    pub imm:        i32,
    pub rs2:        Register,
    pub rs1:        Register,
    pub funct3:     u32,
}

impl From<u32> for InstrS {
    fn from(instr: u32) -> Self {
        let imm115  = instr >> 25;
        let imm40   = (instr >> 07) & 0b0001_1111;

        let imm     = (imm115 << 5) | imm40;
        let imm     = ((imm as i32) << 20) >> 20;

        Self {
            imm,
            rs2: Register::from((instr >> 20) & 0b0001_1111),
            rs1: Register::from((instr >> 15) & 0b0001_1111),
            funct3: (instr >> 12) & 0b0111
        }
    }
}

///       31     30        25  24    20  19    15  14       12  11     8      7     6        0
///  -----------------------------------------------------------------------------------------
/// |   imm12   |   imm105   |   rs2   |   rs1   |   funct3   |  imm40  |  imm11  |  opcode   |
///  -----------------------------------------------------------------------------------------
pub struct InstrB {
    pub imm:        i32,
    pub rs2:        Register,
    pub rs1:        Register,
    pub funct3:     u32,
}

impl From<u32> for InstrB {
    fn from(instr: u32) -> Self {
        let imm11   = (instr >> 07) & 1;
        let imm41   = (instr >> 08) & 0b1111;
        let imm105  = (instr >> 25) & 0b0011_1111;
        let imm12   = (instr >> 31) & 1;

        let imm = (imm12 << 12) | (imm11 << 11) | (imm105 << 5) | (imm41 << 1);
        let imm = ((imm as i32) << 19) >> 19;

        Self {
            imm,
            rs2: Register::from((instr >> 20) & 0b0001_1111),
            rs1: Register::from((instr >> 15) & 0b0001_1111),
            funct3: (instr >> 12) & 0b0111
        }
    }
}

///  31                                                     12  11    7  6         0
///  -------------------------------------------------------------------------------
/// |                         imm3112                         |   rd   |   opcode   |
///  -------------------------------------------------------------------------------
#[derive(Debug)]
pub struct InstrU {
    pub imm:        i32,
    pub rd:         Register,
}

impl From<u32> for InstrU {
    fn from(instr: u32) -> Self {
        Self {
            imm: (instr & !0xfff) as i32,
            rd: Register::from((instr >> 7) & 0b0001_1111)
        }
    }
}

///       31     30        21     20    19                  12  11    7  6         0
///  -------------------------------------------------------------------------------
/// |   imm20   |   imm101   |  imm11  |        imm1912       |   rd   |   opcode   |
///  -------------------------------------------------------------------------------
#[derive(Debug)]
pub struct InstrJ {
    pub imm:        i32,
    pub rd:         Register,
}

impl From<u32> for InstrJ {
    fn from(instr: u32) -> Self {
        let imm1912 = (instr >> 12) & 0b1111_1111;
        let imm11   = (instr >> 20) & 1;
        let imm101  = (instr >> 21) & 0b0011_1111_1111;
        let imm20   = (instr >> 31) & 1;

        let imm     = (imm20 << 20) | (imm1912 << 12) | (imm11 << 11) | (imm101 << 1);
        let imm     = ((imm as i32) << 11) >> 11;

        Self {
            imm,
            rd: Register::from((instr >> 7) & 0b0001_1111)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
