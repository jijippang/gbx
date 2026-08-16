use super::TCycles;
use super::gb_mmu::{Address, Data, GbMmu};
use instruction::Instruction;
use tracing::{error, info, warn};

mod instruction;

#[derive(Debug, PartialEq)]
enum R8 {
    // Accumulator
    A,

    // General Purpose
    B,

    // General Purpose
    C,

    // General Purpose
    D,

    // General Purpose
    E,

    // Flags
    F,

    // General Purpose
    H,

    // General Purpose
    L,
}

impl R8 {
    fn from_byte(byte: u8) -> Result<Self, String> {
        match byte {
            0b111 => Ok(Self::A),
            0b000 => Ok(Self::B),
            0b001 => Ok(Self::C),
            0b010 => Ok(Self::D),
            0b011 => Ok(Self::E),
            0b100 => Ok(Self::H),
            0b101 => Ok(Self::L),
            _ => Err(format!("Unknown byte: {:#010b}, cannot decode R8", byte)),
        }
    }
}

#[derive(Debug, PartialEq)]
enum R16 {
    AF,
    BC,
    DE,
    HL,
}

impl R16 {
    fn from_byte(byte: u8) -> Result<Self, String> {
        match byte {
            0b00 => Ok(Self::BC),
            0b01 => Ok(Self::DE),
            0b10 => Ok(Self::HL),
            _ => Err(format!("Unknown byte: {:#010b}, cannot decode R16", byte)),
        }
    }
}

#[derive(Debug, PartialEq)]
enum Flag {
    // Zero Flag
    Z,

    // Subtraction Flag
    N,

    // Half Carry Flag
    H,

    // Carry Flag
    C,
}

#[derive(Debug, PartialEq)]
enum BitIdx {
    B0,
    B1,
    B2,
    B3,
    B4,
    B5,
    B6,
    B7,
}

#[derive(Debug, PartialEq)]
enum Condition {
    Z,
    NZ,
    C,
    NC,
}

#[derive(Debug, PartialEq)]
#[repr(u8)]
enum RstVec {
    V0 = 0x00,
    V1 = 0x08,
    V2 = 0x10,
    V3 = 0x18,
    V4 = 0x20,
    V5 = 0x28,
    V6 = 0x30,
    V7 = 0x38,
}

#[derive(Debug, PartialEq)]
#[repr(u8)]
enum IntVec {
    V0 = 0x40,
    V1 = 0x48,
    V2 = 0x50,
    V3 = 0x58,
    V4 = 0x60,
}

#[derive(Debug, Default)]
struct Registers {
    // Accumulator
    a: Data,

    // General Purpose
    b: Data,

    // General Purpose
    c: Data,

    // General Purpose
    d: Data,

    // General Purpose
    e: Data,

    // Flags
    f: Data,

    // General Purpose
    h: Data,

    // General Purpose
    l: Data,
}

impl Registers {
    fn get_r8(&self, r8: R8) -> Data {
        match r8 {
            R8::A => self.a,
            R8::B => self.b,
            R8::C => self.c,
            R8::D => self.d,
            R8::E => self.e,
            R8::F => self.f,
            R8::H => self.h,
            R8::L => self.l,
        }
    }

    fn set_r8(&mut self, r8: R8, val: Data) {
        match r8 {
            R8::A => self.a = val,
            R8::B => self.b = val,
            R8::C => self.c = val,
            R8::D => self.d = val,
            R8::E => self.e = val,
            // Only the upper 4 bits of register F are ever set
            R8::F => self.f = val & 0xF0,
            R8::H => self.h = val,
            R8::L => self.l = val,
        }
    }

    fn get_r16(&self, r16: R16) -> u16 {
        match r16 {
            R16::AF => Self::combine_bytes(self.f, self.a),
            R16::BC => Self::combine_bytes(self.c, self.b),
            R16::DE => Self::combine_bytes(self.e, self.d),
            R16::HL => Self::combine_bytes(self.l, self.h),
        }
    }

    fn set_r16(&mut self, r16: R16, val: u16) {
        let (low, high) = Self::split_bytes(val);
        match r16 {
            R16::AF => {
                self.set_r8(R8::F, low);
                self.set_r8(R8::A, high);
            }
            R16::BC => {
                self.set_r8(R8::C, low);
                self.set_r8(R8::B, high);
            }
            R16::DE => {
                self.set_r8(R8::E, low);
                self.set_r8(R8::D, high);
            }
            R16::HL => {
                self.set_r8(R8::L, low);
                self.set_r8(R8::H, high);
            }
        }
    }

    fn get_flag(&self, flag: Flag) -> bool {
        match flag {
            Flag::Z => (self.f & 0x80) != 0,
            Flag::N => (self.f & 0x40) != 0,
            Flag::H => (self.f & 0x20) != 0,
            Flag::C => (self.f & 0x10) != 0,
        }
    }

    fn set_flag(&mut self, flag: Flag, val: bool) {
        match flag {
            Flag::Z => {
                if val {
                    self.f |= 0x80
                } else {
                    self.f &= 0x70
                }
            }
            Flag::N => {
                if val {
                    self.f |= 0x40
                } else {
                    self.f &= 0xB0
                }
            }
            Flag::H => {
                if val {
                    self.f |= 0x20
                } else {
                    self.f &= 0xD0
                }
            }
            Flag::C => {
                if val {
                    self.f |= 0x10
                } else {
                    self.f &= 0xE0
                }
            }
        }
    }

    fn combine_bytes(low: u8, high: u8) -> u16 {
        ((high as u16) << 8) | (low as u16)
    }

    fn split_bytes(bytes: u16) -> (u8, u8) {
        let high = (bytes >> 8) as u8;
        let low = (bytes & 0xFF) as u8;
        (low, high)
    }
}

#[derive(Debug, Default)]
pub struct GbCpu {
    // 16-bit address space with a data size of 8-bits or in other words 64 KiB of memory to read and/or write data
    memory: GbMmu,

    // 8, 8-bit registers which can be combined into 4, 16-bit registers if needed
    registers: Registers,

    // Stack Pointer
    sp: Address,

    // Program Counter
    pc: Address,

    // Interrupt Master Enable Flag
    ime: bool,
}

impl GbCpu {
    pub fn step(&mut self) -> TCycles {
        // Fetch
        let opcode = self.fetch();

        // Decode
        match Instruction::decode(opcode, self) {
            Ok(instr) => {
                // Execute
                self.execute(instr)
            }
            Err(err) => {
                error!("Decode Error: {}", err);

                // TODO: Halt the CPU or jump to a reset vector
                0
            }
        }
    }

    fn fetch(&mut self) -> Data {
        let data = self.memory.read(self.pc);
        self.pc = self.pc.wrapping_add(1);
        data.unwrap_or_else(|err| err.into())
    }

    #[inline]
    fn execute(&mut self, instr: Instruction) -> TCycles {
        // The instruction should already contain both the opcode and any operands
        // And then at this point we can do some kind of function dispatch to execute the instruction
        match instr {
            Instruction::LdR8FromR8 { dst, src } => {
                if dst != src {
                    let src_data = self.registers.get_r8(src);
                    self.registers.set_r8(dst, src_data);
                }
                4
            }

            Instruction::Nop => 4,

            // NOTE: Placeholder, remove once all instructions are implemented
            _ => 0,
        }
    }

    fn check_condition(&self, cond: Condition) -> bool {
        match cond {
            Condition::Z => self.registers.get_flag(Flag::Z) == true,
            Condition::NZ => self.registers.get_flag(Flag::Z) == false,
            Condition::C => self.registers.get_flag(Flag::C) == true,
            Condition::NC => self.registers.get_flag(Flag::C) == false,
        }
    }

    // TODO: Add more helper methods here as needed to aid in the implementation of the execute() call
}

// --- UNIT TESTS BEGIN ---

#[cfg(test)]
mod tests {
    use super::*;

    mod r8_tests {
        use super::*;

        #[test]
        fn test_from_byte() {
            let byte = 0b11111_111;
            let r8 = R8::from_byte(byte);
            assert!(r8.is_err());

            let byte = 0b00000_111;
            let r8 = R8::from_byte(byte).unwrap();
            assert_eq!(r8, R8::A);

            let byte = 0b00000_000;
            let r8 = R8::from_byte(byte).unwrap();
            assert_eq!(r8, R8::B);

            let byte = 0b00000_001;
            let r8 = R8::from_byte(byte).unwrap();
            assert_eq!(r8, R8::C);

            let byte = 0b00000_010;
            let r8 = R8::from_byte(byte).unwrap();
            assert_eq!(r8, R8::D);

            let byte = 0b00000_011;
            let r8 = R8::from_byte(byte).unwrap();
            assert_eq!(r8, R8::E);

            let byte = 0b00000_100;
            let r8 = R8::from_byte(byte).unwrap();
            assert_eq!(r8, R8::H);

            let byte = 0b00000_101;
            let r8 = R8::from_byte(byte).unwrap();
            assert_eq!(r8, R8::L);
        }
    }

    mod r16_tests {
        use super::*;

        #[test]
        fn test_from_byte() {
            let byte = 0b111111_11;
            let r16 = R16::from_byte(byte);
            assert!(r16.is_err());

            let byte = 0b000000_00;
            let r16 = R16::from_byte(byte).unwrap();
            assert_eq!(r16, R16::BC);

            let byte = 0b000000_01;
            let r16 = R16::from_byte(byte).unwrap();
            assert_eq!(r16, R16::DE);

            let byte = 0b000000_10;
            let r16 = R16::from_byte(byte).unwrap();
            assert_eq!(r16, R16::HL);
        }
    }

    mod registers_tests {
        use super::*;

        #[test]
        fn test_get_r8() {
            let registers = Registers {
                a: 0x4F,
                b: 0x72,
                c: 0xFF,
                d: 0x28,
                e: 0x92,
                f: 0xD9,
                h: 0xE9,
                l: 0xA7,
            };

            let result = registers.get_r8(R8::A);
            assert_eq!(result, 0x4F);

            let result = registers.get_r8(R8::B);
            assert_eq!(result, 0x72);

            let result = registers.get_r8(R8::C);
            assert_eq!(result, 0xFF);

            let result = registers.get_r8(R8::D);
            assert_eq!(result, 0x28);

            let result = registers.get_r8(R8::E);
            assert_eq!(result, 0x92);

            let result = registers.get_r8(R8::F);
            assert_eq!(result, 0xD9);

            let result = registers.get_r8(R8::H);
            assert_eq!(result, 0xE9);

            let result = registers.get_r8(R8::L);
            assert_eq!(result, 0xA7);
        }

        #[test]
        fn test_set_r8() {
            let mut registers = Registers::default();

            registers.set_r8(R8::A, 0x4F);
            assert_eq!(registers.get_r8(R8::A), 0x4F);

            registers.set_r8(R8::B, 0x72);
            assert_eq!(registers.get_r8(R8::B), 0x72);

            registers.set_r8(R8::C, 0xFF);
            assert_eq!(registers.get_r8(R8::C), 0xFF);

            registers.set_r8(R8::D, 0x28);
            assert_eq!(registers.get_r8(R8::D), 0x28);

            registers.set_r8(R8::E, 0x92);
            assert_eq!(registers.get_r8(R8::E), 0x92);

            registers.set_r8(R8::F, 0xD9);
            assert_eq!(registers.get_r8(R8::F), 0xD0);

            registers.set_r8(R8::H, 0xE9);
            assert_eq!(registers.get_r8(R8::H), 0xE9);

            registers.set_r8(R8::L, 0xA7);
            assert_eq!(registers.get_r8(R8::L), 0xA7);
        }

        #[test]
        fn test_get_r16() {
            let registers = Registers {
                a: 0x4F,
                b: 0x72,
                c: 0xFF,
                d: 0x28,
                e: 0x92,
                f: 0xD9,
                h: 0xE9,
                l: 0xA7,
            };

            let result = registers.get_r16(R16::AF);
            assert_eq!(result, 0x4FD9);

            let result = registers.get_r16(R16::BC);
            assert_eq!(result, 0x72FF);

            let result = registers.get_r16(R16::DE);
            assert_eq!(result, 0x2892);

            let result = registers.get_r16(R16::HL);
            assert_eq!(result, 0xE9A7);
        }

        #[test]
        fn test_set_r16() {
            let mut registers = Registers::default();

            registers.set_r16(R16::AF, 0x4FD9);
            assert_eq!(registers.get_r16(R16::AF), 0x4FD0);

            registers.set_r16(R16::BC, 0x72FF);
            assert_eq!(registers.get_r16(R16::BC), 0x72FF);

            registers.set_r16(R16::DE, 0x2892);
            assert_eq!(registers.get_r16(R16::DE), 0x2892);

            registers.set_r16(R16::HL, 0xE9A7);
            assert_eq!(registers.get_r16(R16::HL), 0xE9A7);
        }

        #[test]
        fn test_get_flag() {
            let registers = Registers {
                f: 0xD9,
                ..Default::default()
            };

            let result = registers.get_flag(Flag::Z);
            assert!(result);

            let result = registers.get_flag(Flag::N);
            assert!(result);

            let result = registers.get_flag(Flag::H);
            assert!(!result);

            let result = registers.get_flag(Flag::C);
            assert!(result);
        }

        #[test]
        fn test_set_flag() {
            let mut registers = Registers::default();

            registers.set_flag(Flag::Z, false);
            assert!(!registers.get_flag(Flag::Z));

            registers.set_flag(Flag::N, true);
            assert!(registers.get_flag(Flag::N));

            registers.set_flag(Flag::H, true);
            assert!(registers.get_flag(Flag::H));

            registers.set_flag(Flag::C, false);
            assert!(!registers.get_flag(Flag::C));
        }
    }

    mod gb_cpu_tests {
        use super::*;

        #[test]
        fn test() {
            // TODO: Add unit tests for GbCpu
        }
    }
}

// --- UNIT TESTS END ---
