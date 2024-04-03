use core::convert::TryFrom;

#[cfg(not(feature = "std"))]
use core::{fmt::Write, result};

#[cfg(feature = "std")]
use std::{
    io::{self, Write},
    result,
};

pub const MEMORY_SIZE: usize = 4096;
const NREGS: usize = 16;

const IP: usize = 0;

type Result<T, E = Error> = result::Result<T, E>;

pub struct Machine {
    memory: [u8; MEMORY_SIZE],
    registers: [u32; NREGS],
}

#[derive(Debug)]
enum Instruction {
    MoveIf {
        target: usize,
        source: usize,
        cond: usize,
    },
    Load {
        target: usize,
        source: usize,
    },
    Store {
        target: usize,
        source: usize,
    },
    LoadImm {
        target: usize,
        value: i32,
    },
    Sub {
        target: usize,
        op1: usize,
        op2: usize,
    },
    Out {
        reg: usize,
    },
    OutNumber {
        reg: usize,
    },
    Exit,
}

impl Instruction {
    fn to_reg(r: u8) -> Result<usize> {
        match r as usize {
            r if r < NREGS => Ok(r),
            r => Err(Error::InvalidRegister(r)),
        }
    }

    fn to_imm(l: u8, h: u8) -> i32 {
        i32::from(i16::from_le_bytes([l, h]))
    }

    fn size(&self) -> usize {
        match self {
            Self::MoveIf { .. } | Self::LoadImm { .. } | Self::Sub { .. } => 4,
            Self::Store { .. } | Self::Load { .. } => 3,
            Self::Out { .. } | Self::OutNumber { .. } => 2,
            Self::Exit => 1,
        }
    }
}

impl TryFrom<&[u8]> for Instruction {
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<Self> {
        let byte = |index| bytes.get(index).copied().ok_or(Error::ReadPastMemoryEnd);
        let op = match byte(0)? {
            1 => Self::MoveIf {
                target: Instruction::to_reg(byte(1)?)?,
                source: Instruction::to_reg(byte(2)?)?,
                cond: Instruction::to_reg(byte(3)?)?,
            },
            2 => Self::Store {
                target: Instruction::to_reg(byte(1)?)?,
                source: Instruction::to_reg(byte(2)?)?,
            },
            3 => Self::Load {
                target: Instruction::to_reg(byte(1)?)?,
                source: Instruction::to_reg(byte(2)?)?,
            },
            4 => Self::LoadImm {
                target: Instruction::to_reg(byte(1)?)?,
                value: Instruction::to_imm(byte(2)?, byte(3)?),
            },
            5 => Self::Sub {
                target: Instruction::to_reg(byte(1)?)?,
                op1: Instruction::to_reg(byte(2)?)?,
                op2: Instruction::to_reg(byte(3)?)?,
            },
            6 => Self::Out {
                reg: Instruction::to_reg(byte(1)?)?,
            },
            7 => Self::Exit,
            8 => Self::OutNumber {
                reg: Instruction::to_reg(byte(1)?)?,
            },
            o => return Err(Error::UnknownOpcode(o)),
        };
        Ok(op)
    }
}

#[derive(Debug)]
pub enum Error {
    /// Attempt to create a machine with too large a memory
    MemoryOverflow,
    UnknownOpcode(u8),
    InvalidRegister(usize),
    InvalidMemoryAddress(u32),
    ReadPastMemoryEnd,
    OutputError,
}

impl Machine {
    /// Create a new machine in its reset state. The `memory` parameter will
    /// be copied at the beginning of the machine memory.
    ///
    /// # Errors
    /// This function returns an error when the memory exceeds `MEMORY_SIZE`.
    pub fn new(memory: &[u8]) -> Result<Self> {
        if memory.len() > MEMORY_SIZE {
            return Err(Error::MemoryOverflow);
        }
        let mut machine = Self {
            memory: [0; MEMORY_SIZE],
            registers: [0; NREGS],
        };
        machine.memory[..memory.len()].copy_from_slice(memory);
        Ok(machine)
    }

    /// Run until the program terminates or until an error happens.
    /// If output instructions are run, they print on `fd`.
    pub fn run_on<T: Write>(&mut self, fd: &mut T) -> Result<()> {
        while !self.step_on(fd)? {}
        Ok(())
    }

    /// Run until the program terminates or until an error happens.
    /// If output instructions are run, they print on standard output.
    #[cfg(feature = "std")]
    pub fn run(&mut self) -> Result<()> {
        self.run_on(&mut io::stdout().lock())
    }

    /// Execute the next instruction by doing the following steps:
    ///   - decode the instruction located at IP (register 0)
    ///   - increment the IP by the size of the instruction
    ///   - execute the decoded instruction
    ///
    /// If output instructions are run, they print on `fd`.
    /// If an error happens at either of those steps, an error is
    /// returned.
    ///
    /// In case of success, `true` is returned if the program is
    /// terminated (upon encountering an exit instruction), or
    /// `false` if the execution must continue.
    pub fn step_on<T: Write>(&mut self, fd: &mut T) -> Result<bool> {
        let ip = self.registers[IP] as usize;
        let instruction = Instruction::try_from(self.memory.get(ip..).unwrap_or_default())?;
        self.registers[IP] += instruction.size() as u32;
        self.execute_instruction(instruction, fd)
    }

    /// Similar to [`step_on`](Machine::step_on).
    /// If output instructions are run, they print on standard output.
    #[cfg(feature = "std")]
    pub fn step(&mut self) -> Result<bool> {
        self.step_on(&mut io::stdout().lock())
    }

    /// Reference onto the machine current set of registers.
    #[must_use]
    pub fn regs(&self) -> &[u32] {
        &self.registers
    }

    /// Sets a register to the given value.
    pub fn set_reg(&mut self, reg: usize, value: u32) -> Result<()> {
        match self.registers.get_mut(reg) {
            Some(r) => Ok(*r = value),
            None => Err(Error::InvalidRegister(reg)),
        }
    }

    /// Reference onto the machine current memory.
    #[must_use]
    pub fn memory(&self) -> &[u8] {
        &self.memory
    }

    fn execute_instruction<T: Write>(
        &mut self,
        instruction: Instruction,
        fd: &mut T,
    ) -> Result<bool> {
        match instruction {
            Instruction::MoveIf {
                target,
                source,
                cond,
            } => {
                if self.registers[cond] != 0 {
                    self.registers[target] = self.registers[source];
                }
            }
            Instruction::Load { target, source } => {
                self.registers[target] = self.get_memory_u32(self.registers[source])?;
            }
            Instruction::Store { target, source } => {
                self.store_memory(self.registers[target], self.registers[source])?;
            }
            Instruction::LoadImm { target, value } => {
                self.registers[target] = value as u32;
            }
            Instruction::Sub { target, op1, op2 } => {
                self.registers[target] = self.registers[op1].wrapping_sub(self.registers[op2]);
            }
            Instruction::Out { reg } => {
                write!(fd, "{}", self.registers[reg] as u8 as char)
                    .map_err(|_| Error::OutputError)?;
            }
            Instruction::Exit => return Ok(true),
            Instruction::OutNumber { reg } => {
                write!(fd, "{}", self.registers[reg] as i32).map_err(|_| Error::OutputError)?;
            }
        }
        Ok(false)
    }

    fn get_memory_address(addr: u32) -> Result<usize> {
        if addr < MEMORY_SIZE as u32 {
            Ok(addr as usize)
        } else {
            Err(Error::InvalidMemoryAddress(addr))
        }
    }

    fn get_memory(&self, addr: u32) -> Result<u8> {
        Ok(self.memory[Self::get_memory_address(addr)?])
    }

    fn get_memory_u32(&self, addr: u32) -> Result<u32> {
        Ok(u32::from_le_bytes([
            self.get_memory(addr)?,
            self.get_memory(addr + 1)?,
            self.get_memory(addr + 2)?,
            self.get_memory(addr + 3)?,
        ]))
    }

    fn store_memory(&mut self, addr: u32, value: u32) -> Result<()> {
        let bytes = value.to_le_bytes();
        for i in 0..4 {
            self.memory[Self::get_memory_address(addr + i)?] = bytes[i as usize];
        }
        Ok(())
    }
}
