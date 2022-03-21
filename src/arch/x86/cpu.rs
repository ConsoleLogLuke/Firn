use crate::arch::x86::SegmentReg::{Cs, Ds, Es, Ss};
use crate::arch::x86::{GeneralByteReg, Instr, SegmentReg, WordReg};
use crate::{cpu, System};

pub struct Cpu {
    pub system: System,

    regs: [u8; 2 * 8],
    segments: [u16; 4],

    pub flags: u16,
    pub ip: u16,
}

impl Cpu {
    pub fn new(system: System) -> Self {
        Self {
            system,
            regs: [0; 2 * 8],
            segments: [0; 4],
            flags: 0,
            ip: 0,
        }
    }

    pub fn get_reg_8(&self, reg: GeneralByteReg) -> u8 {
        self.regs[reg as usize]
    }

    pub fn get_reg_16(&self, reg: WordReg) -> u16 {
        match reg {
            WordReg::General(reg) => {
                let low = self.regs[reg as usize];
                let high = self.regs[reg as usize + 4];

                u16::from_le_bytes([low, high])
            }
            WordReg::Segment(reg) => self.segments[reg as usize],
        }
    }

    pub fn set_reg_8(&mut self, reg: GeneralByteReg, value: u8) {
        self.regs[reg as usize] = value;
    }

    pub fn set_reg_16(&mut self, reg: WordReg, value: u16) {
        match reg {
            WordReg::General(reg) => {
                let [low, high] = value.to_le_bytes();

                self.regs[reg as usize] = low;
                self.regs[reg as usize + 4] = high;
            }
            WordReg::Segment(reg) => self.segments[reg as usize] = value,
        };
    }

    pub fn get_mem_8(&self, segment: SegmentReg, offset: u16) -> u8 {
        let linear = self.linear_mem(segment, offset);

        self.system.mem[linear]
    }

    pub fn get_mem_16(&self, segment: SegmentReg, offset: u16) -> u16 {
        let linear = self.linear_mem(segment, offset);

        let low = self.system.mem[linear];
        let high = self.system.mem[linear + 1];

        u16::from_le_bytes([low, high])
    }

    pub fn set_mem_8(&mut self, segment: SegmentReg, offset: u16, value: u8) {
        let linear = self.linear_mem(segment, offset);
        self.system.mem[linear] = value;
    }

    pub fn set_mem_16(&mut self, segment: SegmentReg, offset: u16, value: u16) {
        let linear = self.linear_mem(segment, offset);

        let [low, high] = value.to_le_bytes();
        self.system.mem[linear] = low;
        self.system.mem[linear + 1] = high;
    }

    pub fn read_mem_8(&mut self) -> u8 {
        let value = self.get_mem_8(Cs, self.ip);
        self.ip += 1;

        value
    }

    pub fn read_mem_16(&mut self) -> u16 {
        let value = self.get_mem_16(Cs, self.ip);
        self.ip += 2;

        value
    }

    fn linear_mem(&self, segment: SegmentReg, offset: u16) -> usize {
        let segment = self.get_reg_16(segment.into()) as usize;

        (segment << 4) + offset as usize
    }
}

impl cpu::Cpu for Cpu {
    fn reset(&mut self) {
        self.set_reg_16(Cs.into(), 0xffff);
        self.set_reg_16(Ds.into(), 0x0000);
        self.set_reg_16(Es.into(), 0x0000);
        self.set_reg_16(Ss.into(), 0x0000);

        self.ip = 0;
    }

    fn step(&mut self) {
        let instr = Instr::decode(self);
        println!("Decoded: {:?}", instr);
        instr.execute(self);
    }
}
