struct Cpu {
    regs: [u32; 32],
    pc: u32,
}

impl Cpu {
    fn try_handle_insn(&mut self, insn: &Insn) -> Result<(), InsnError> {
        match insn {
            Insn::Reg(i) => self.try_handle_insn_reg(i),
            Insn::Imm(i) => self.try_handle_insn_imm(i),
        }
    }

    fn try_handle_insn_reg(&mut self, insn: &RegInsn) -> Result<(), InsnError> {
        if let Reg::Zero = insn.rd {
            return Err(InsnError::DstRegZeroError);
        }

        let rs_u: u32 = self.regs[insn.rs as usize];
        let rt_u: u32 = self.regs[insn.rt as usize];
        let rd_u: &mut u32 = &mut self.regs[insn.rd as usize];

        let rs_i: i32 = unsafe { std::mem::transmute_copy(&rs_u) };
        let rt_i: i32 = unsafe { std::mem::transmute_copy(&rt_u) };
        let rd_i: &mut i32 = unsafe { std::mem::transmute_copy(&rd_u) };

        match &insn.funct {
            RegFunc::Sll => *rd_u = rt_u << insn.shamt,
            RegFunc::SllV => *rd_u = rt_u << rs_u,
            RegFunc::Srl => *rd_u = rt_u >> insn.shamt,
            RegFunc::SrlV => *rd_u = rt_u >> rs_u,
            RegFunc::Sra => *rd_i = rt_i >> insn.shamt,
            RegFunc::SraV => *rd_i = rt_i >> rs_i,
            RegFunc::Add => {
                let (val, overflow) = rs_i.overflowing_add(rt_i);
                *rd_i = val;
                if overflow {
                    return Err(InsnError::IntegerOverflowException);
                }
            }
            RegFunc::AddU => {
                let (val, _overflow) = rs_u.overflowing_add(rt_u);
                *rd_u = val;
            }
            RegFunc::Sub => {
                let (val, overflow) = rs_i.overflowing_sub(rt_i);
                *rd_i = val;
                if overflow {
                    return Err(InsnError::IntegerOverflowException);
                }
            }
            RegFunc::SubU => {
                let (val, _overflow) = rs_u.overflowing_sub(rt_u);
                *rd_u = val;
            }
            RegFunc::And => *rd_u = rs_u & rt_u,
            RegFunc::Or => *rd_u = rs_u | rt_u,
            RegFunc::Xor => *rd_u = rs_u ^ rt_u,
            RegFunc::Nor => *rd_u = !(rs_u | rt_u),
        }

        Ok(())
    }

    fn try_handle_insn_imm(&mut self, insn: &ImmInsn) -> Result<(), InsnError> {
        let rs_u: u32 = self.regs[insn.rs as usize];
        let rt_u: &mut u32 = &mut self.regs[insn.rt as usize];

        let rs_i: i32 = unsafe { std::mem::transmute_copy(&rs_u) };
        let rt_i: &mut i32 = unsafe { std::mem::transmute_copy(&rt_u) };

        match &insn.opcode {
            ImmOpcode::AddI => {
                let (val, overflow) = rs_i.overflowing_add(insn.data as i32);
                *rt_i = val;
                if overflow {
                    return Err(InsnError::IntegerOverflowException);
                }
            }
            ImmOpcode::AddIU => {
                let (val, overflow) = rs_u.overflowing_add(insn.data as u32);
                *rt_u = val;
                if overflow {
                    return Err(InsnError::IntegerOverflowException);
                }
            }
            ImmOpcode::AndI => *rt_u = rs_u & (insn.data as u32),
            ImmOpcode::OrI => *rt_u = rs_u | (insn.data as u32),
            ImmOpcode::XorI => *rt_u = rs_u ^ (insn.data as u32),
            ImmOpcode::LuI => *rt_u = (insn.data as u32) << 16,
        }

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
enum InsnError {
    #[error("integer overflow")]
    IntegerOverflowException,

    #[error("destination register is $zero")]
    DstRegZeroError,
}

#[derive(Debug)]
enum Insn {
    Reg(RegInsn),
    Imm(ImmInsn),
}

#[derive(Debug)]
struct RegInsn {
    rs: Reg,
    rt: Reg,
    rd: Reg,
    shamt: u8,
    funct: RegFunc,
}

#[derive(Debug, Copy, Clone, num_enum::TryFromPrimitive, num_enum::IntoPrimitive)]
#[repr(u8)]
enum RegFunc {
    Sll = 0b00_0000,
    SllV = 0b00_0100,
    Srl = 0b00_0010,
    SrlV = 0b00_0110,
    Sra = 0b00_0011,
    SraV = 0b00_0111,
    Add = 0b10_0000,
    AddU = 0b10_0001,
    Sub = 0b10_0010,
    SubU = 0b10_0011,
    And = 0b10_0100,
    Or = 0b10_0101,
    Xor = 0b10_0110,
    Nor = 0b10_0111,
}

#[derive(Debug)]
struct ImmInsn {
    opcode: ImmOpcode,
    rs: Reg,
    rt: Reg,
    data: u16,
}

#[derive(Debug, Copy, Clone, num_enum::TryFromPrimitive, num_enum::IntoPrimitive)]
#[repr(u8)]
enum ImmOpcode {
    AddI = 0b00_1000,
    AddIU = 0b00_1001,
    AndI = 0b00_1100,
    OrI = 0b00_1101,
    XorI = 0b00_1110,
    LuI = 0b00_1111,
}

#[derive(Debug, Copy, Clone, num_enum::TryFromPrimitive, num_enum::IntoPrimitive)]
#[repr(u8)]
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
