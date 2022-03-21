use crate::arch::x86::GeneralWordReg::{Bp, Bx, Di, Si};
use crate::arch::x86::SegmentReg::{Ds, Ss};
use crate::arch::x86::{Cpu, GeneralByteReg, GeneralReg, GeneralWordReg, Reg, SegmentReg, Size};

#[derive(Debug, Copy, Clone)]
pub enum ModrmRegType {
    ByteSized,
    WordSized,
    Segment,
}

#[derive(Debug)]
pub enum Displacement {
    SignedByte(i8),
    UnsignedWord(u16),
}

#[derive(Debug)]
pub struct RmPtr {
    segment: SegmentReg,
    first_reg: Option<GeneralWordReg>,
    second_reg: Option<GeneralWordReg>,
    displacement: Option<Displacement>,
}

impl RmPtr {
    pub fn to_linear(&self, cpu: &Cpu) -> (SegmentReg, u16) {
        let mut offset: u16 = 0;

        for reg in [self.first_reg, self.second_reg].into_iter().flatten() {
            let value = cpu.get_reg_16(reg.into());
            offset = offset.wrapping_add(value);
        }

        let displacement = match self.displacement {
            Some(Displacement::SignedByte(displacement)) => displacement as u16,
            Some(Displacement::UnsignedWord(displacement)) => displacement,
            None => 0,
        };

        (self.segment, offset + displacement)
    }
}

#[derive(Debug)]
pub enum RegMem {
    Reg(GeneralReg),
    Ptr(RmPtr),
}

impl RegMem {
    pub fn get_8(&self, cpu: &Cpu) -> u8 {
        match self {
            RegMem::Reg(reg) => match reg {
                GeneralReg::Byte(reg) => cpu.get_reg_8(*reg),
                _ => panic!("cannot get a byte-sized value from a non-byte-sized RM"),
            },
            RegMem::Ptr(ptr) => {
                let (segment, offset) = ptr.to_linear(cpu);
                cpu.get_mem_8(segment, offset)
            }
        }
    }

    pub fn get_16(&self, cpu: &Cpu) -> u16 {
        match self {
            RegMem::Reg(reg) => match reg {
                GeneralReg::Word(reg) => cpu.get_reg_16((*reg).into()),
                _ => panic!("cannot get a word-sized value from a non-word-sized RM"),
            },
            RegMem::Ptr(ptr) => {
                let (segment, offset) = ptr.to_linear(cpu);
                cpu.get_mem_16(segment, offset)
            }
        }
    }

    pub fn set_8(&self, cpu: &mut Cpu, value: u8) {
        match self {
            RegMem::Reg(reg) => match reg {
                GeneralReg::Byte(reg) => cpu.set_reg_8(*reg, value),
                _ => panic!("cannot set a byte-sized value to a non-byte-sized RM"),
            },
            RegMem::Ptr(ptr) => {
                let (segment, offset) = ptr.to_linear(cpu);
                cpu.set_mem_8(segment, offset, value);
            }
        }
    }

    pub fn set_16(&self, cpu: &mut Cpu, value: u16) {
        match self {
            RegMem::Reg(reg) => match reg {
                GeneralReg::Word(reg) => cpu.set_reg_16((*reg).into(), value),
                _ => panic!("cannot set a word-sized value to a non-word-sized RM"),
            },
            RegMem::Ptr(ptr) => {
                let (segment, offset) = ptr.to_linear(cpu);
                cpu.set_mem_16(segment, offset, value);
            }
        }
    }
}

#[derive(Debug)]
pub struct Modrm {
    pub reg: Option<Reg>,
    pub reg_mem: RegMem,
}

impl Modrm {
    pub fn decode(cpu: &mut Cpu, modrm: u8, reg_type: Option<ModrmRegType>, rm_size: Size) -> Self {
        let x = (modrm / 0o100) % 0o10;
        let r = (modrm / 0o10) % 0o10;
        let m = modrm % 0o10;

        let reg = reg_type.map(|reg_type| match reg_type {
            ModrmRegType::ByteSized => match GeneralByteReg::from_u8(r) {
                Some(reg) => reg.into(),
                None => panic!("invalid r (in xrm octal) in ModRM byte: {}", r),
            },
            ModrmRegType::WordSized => match GeneralWordReg::from_u8(r) {
                Some(reg) => reg.into(),
                None => panic!("invalid r (in xrm octal) in ModRM byte: {}", r),
            },
            ModrmRegType::Segment => match SegmentReg::from_u8(r) {
                Some(reg) => reg.into(),
                None => panic!("invalid s (in xsm octal) in ModRM byte: {}", r),
            },
        });

        if x == 3 {
            let rm_reg = match rm_size {
                Size::Byte => match GeneralByteReg::from_u8(m) {
                    Some(reg) => reg.into(),
                    None => panic!("invalid m (in 3rm octal) in ModRM byte: {}", m),
                },
                Size::Word => match GeneralWordReg::from_u8(m) {
                    Some(reg) => reg.into(),
                    None => panic!("invalid m (in 3rm octal) in ModRM byte: {}", m),
                },
            };

            return Modrm {
                reg,
                reg_mem: RegMem::Reg(rm_reg),
            };
        }

        let displacement = match x {
            0 if m == 6 => Some(Displacement::UnsignedWord(cpu.read_mem_16())),
            0 => None,
            1 => Some(Displacement::SignedByte(cpu.read_mem_8() as i8)),
            2 => Some(Displacement::UnsignedWord(cpu.read_mem_16())),

            _ => panic!("invalid x (in xrm octal) in ModRM byte: {}", x),
        };

        let (segment, first_reg, second_reg) = match m {
            0 => (Ds, Some(Bx), Some(Si)),
            1 => (Ds, Some(Bx), Some(Di)),
            2 => (Ss, Some(Bp), Some(Si)),
            3 => (Ss, Some(Bp), Some(Di)),
            4 => (Ds, Some(Si), None),
            5 => (Ds, Some(Di), None),
            6 if x == 0 => (Ds, None, None),
            6 => (Ss, Some(Bp), None),
            7 => (Ds, Some(Bx), None),

            _ => panic!("invalid m (in xrm octal) in ModRM byte: {}", m),
        };

        Modrm {
            reg,
            reg_mem: RegMem::Ptr(RmPtr {
                segment,
                first_reg,
                second_reg,
                displacement,
            }),
        }
    }
}
