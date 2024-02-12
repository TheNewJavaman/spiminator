use num_enum::{TryFromPrimitive, UnsafeFromPrimitive};
use std::collections::HashMap;
use std::mem::transmute;

#[derive(Debug)]
struct Computer {
    regs: [u32; 32],
    program: Vec<Insn>,
    /// Direct index into `program`, not a byte offset
    pc: usize,
    mem: HashMap<u32, u32>,
}

impl Computer {
    fn run(&mut self) -> Result<(), InsnError> {
        let mut exit = false;
        while self.pc < self.program.len() && !exit {
            self.try_handle_insn(self.program[self.pc], &mut exit)?;
        }
        Ok(())
    }

    fn try_handle_insn(&mut self, insn: Insn, exit: &mut bool) -> Result<(), InsnError> {
        match insn.opcode()? {
            Opcode::Reg => match insn.funct()? {
                Funct::Sll => *self.ru_mut(insn.rd())? = self.ru(insn.rt()) << insn.shamt(),
                Funct::SllV => *self.ru_mut(insn.rd())? = self.ru(insn.rt()) << self.ru(insn.rs()),
                Funct::Srl => *self.ru_mut(insn.rd())? = self.ru(insn.rt()) >> insn.shamt(),
                Funct::SrlV => *self.ru_mut(insn.rd())? = self.ru(insn.rt()) >> self.ru(insn.rs()),
                Funct::Sra => *self.ri_mut(insn.rd())? = self.ri(insn.rt()) >> insn.shamt(),
                Funct::SraV => *self.ri_mut(insn.rd())? = self.ri(insn.rt()) >> self.ri(insn.rs()),
                Funct::Syscall => {
                    let code = SyscallCode::try_from_primitive(self.ru(Reg::V0))
                        .map_err(|e| InsnError::UnsupportedSyscall(e.number))?;
                    match code {
                        SyscallCode::Exit => *exit = true,
                    }
                }
                Funct::Add => {
                    let (val, overflow) = self.ri(insn.rs()).overflowing_add(self.ri(insn.rt()));
                    *self.ri_mut(insn.rd())? = val;
                    if overflow {
                        return Err(InsnError::IntegerOverflow);
                    }
                }
                Funct::AddU => {
                    let (val, _overflow) = self.ru(insn.rs()).overflowing_add(self.ru(insn.rt()));
                    *self.ru_mut(insn.rd()) = val;
                }
                Funct::Sub => {
                    let (val, overflow) = self.ri(insn.rs()).overflowing_sub(self.ri(insn.rt()));
                    *self.ri_mut(insn.rd())? = val;
                    if overflow {
                        return Err(InsnError::IntegerOverflow);
                    }
                }
                Funct::SubU => {
                    let (val, _overflow) = self.ru(insn.rs()).overflowing_sub(self.ru(insn.rt()));
                    *self.ru_mut(insn.rd()) = val;
                }
                Funct::And => *self.ru_mut(insn.rd())? = self.ru(insn.rs()) & self.ru(insn.rt()),
                Funct::Or => *self.ru_mut(insn.rd())? = self.ru(insn.rs()) | self.ru(insn.rt()),
                Funct::Xor => *self.ru_mut(insn.rd())? = self.ru(insn.rs()) ^ self.ru(insn.rt()),
                Funct::Nor => *self.ru_mut(insn.rd())? = !(self.ru(insn.rs()) | self.ru(insn.rt())),
            },
            Opcode::AddI => {
                let (val, overflow) = self.ri(insn.rs()).overflowing_add(insn.di());
                *self.ri_mut(insn.rd())? = val;
                if overflow {
                    return Err(InsnError::IntegerOverflow);
                }
            }
            Opcode::AddIU => {
                let (val, overflow) = self.ru(insn.rs()).overflowing_add(insn.du());
                *self.ru_mut(insn.rd())? = val;
                if overflow {
                    return Err(InsnError::IntegerOverflow);
                }
            }
            Opcode::AndI => *self.ru_mut(insn.rd())? = self.ru(insn.rs()) & insn.du(),
            Opcode::OrI => *self.ru_mut(insn.rd())? = self.ru(insn.rs()) | insn.du(),
            Opcode::XorI => *self.ru_mut(insn.rd())? = self.ru(insn.rs()) ^ insn.du(),
            Opcode::LuI => *self.ru_mut(insn.rd())? = insn.du() << 16,
        }

        self.pc += 1;

        Ok(())
    }

    fn ru(&self, reg: Reg) -> u32 {
        self.regs[reg as usize]
    }

    fn ri(&self, reg: Reg) -> i32 {
        unsafe { transmute(self.regs[reg as usize]) }
    }

    fn ru_mut(&mut self, reg: Reg) -> Result<&mut u32, InsnError> {
        match reg {
            Reg::Zero => Err(InsnError::RegMutZero),
            r => Ok(&mut self.regs[r as usize]),
        }
    }

    fn ri_mut(&mut self, reg: Reg) -> Result<&mut i32, InsnError> {
        match reg {
            Reg::Zero => Err(InsnError::RegMutZero),
            r => unsafe { Ok(transmute(&mut self.regs[r as usize])) },
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Insn(u32);

impl Insn {
    fn opcode(&self) -> Result<Opcode, InsnError> {
        Opcode::try_from_primitive(self.0 >> 26).map_err(|e| InsnError::InvalidOpcode(e.number))
    }

    fn rs(&self) -> Reg {
        unsafe { Reg::unchecked_transmute_from((self.0 >> 21) & 0x1F) }
    }

    fn rt(&self) -> Reg {
        unsafe { Reg::unchecked_transmute_from((self.0 >> 16) & 0x1F) }
    }

    fn rd(&self) -> Reg {
        unsafe { Reg::unchecked_transmute_from((self.0 >> 11) & 0x1F) }
    }

    fn shamt(&self) -> u32 {
        (self.0 >> 6) & 0x1F
    }

    fn funct(&self) -> Result<Funct, InsnError> {
        Funct::try_from_primitive(self.0 & 0x3F).map_err(|e| InsnError::InvalidFunct(e.number))
    }

    fn du(&self) -> u32 {
        self.0 & 0xFFFF
    }

    fn di(&self) -> i32 {
        unsafe { transmute((self.0 & 0xFFFF) | 0xFFFF0000) }
    }

    fn addr(&self) -> u32 {
        self.0 & 0x3FFFFFF
    }
}

#[derive(Copy, num_enum::TryFromPrimitive)]
#[repr(u32)]
enum Opcode {
    Reg = 0b000000,
    AddI = 0b001000,
    AddIU = 0b001001,
    AndI = 0b001100,
    OrI = 0b001101,
    XorI = 0b001110,
    LuI = 0b001111,
}

#[derive(Copy, num_enum::UnsafeFromPrimitive)]
#[repr(u32)]
enum Reg {
    Zero = 0,
    At = 1,
    V0 = 2,
    V1 = 3,
    A0 = 4,
    A1 = 5,
    A2 = 6,
    A3 = 7,
    T0 = 8,
    T1 = 9,
    T2 = 10,
    T3 = 11,
    T4 = 12,
    T5 = 13,
    T6 = 14,
    T7 = 15,
    S0 = 16,
    S1 = 17,
    S2 = 18,
    S3 = 19,
    S4 = 20,
    S5 = 21,
    S6 = 22,
    S7 = 23,
    T8 = 24,
    T9 = 25,
    K0 = 26,
    K1 = 27,
    GP = 28,
    SP = 29,
    FP = 30,
    RA = 31,
}

#[derive(Copy, num_enum::TryFromPrimitive)]
#[repr(u32)]
enum Funct {
    Sll = 0b000000,
    SllV = 0b000100,
    Srl = 0b000010,
    SrlV = 0b000110,
    Sra = 0b000011,
    SraV = 0b000111,
    Syscall = 0b001100,
    Add = 0b100000,
    AddU = 0b100001,
    Sub = 0b100010,
    SubU = 0b100011,
    And = 0b100100,
    Or = 0b100101,
    Xor = 0b100110,
    Nor = 0b100111,
}

#[derive(Copy, num_enum::TryFromPrimitive, num_enum::IntoPrimitive)]
#[repr(u32)]
enum SyscallCode {
    Exit = 10,
}

#[derive(thiserror::Error)]
enum InsnError {
    #[error("integer overflow")]
    IntegerOverflow,

    #[error("attempted to mutate $zero")]
    RegMutZero,

    #[error("invalid opcode {0:#b}")]
    InvalidOpcode(u32),

    #[error("invalid funct {0:#b}")]
    InvalidFunct(u32),

    #[error("unsupported syscall $v0={0}")]
    UnsupportedSyscall(u32),
}
