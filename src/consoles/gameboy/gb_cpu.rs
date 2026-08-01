

const MASTER_CLK_FREQ: f64 = 4.194304e6;





#[derive(Debug)]
enum R8
{
    A, B, C, D, E, H, L,
}


#[derive(Debug)]
enum R16
{
    SP, BC, DE, HL,
}


#[derive(Debug)]
enum Instruction
{
    // --- 8-BIT LOAD INSTRUCTIONS BEGIN ---


    // LD r, r': Load register (register)
    // Description: Load to the 8-bit register r, data from the 8-bit register r'
    // Opcode: 0b01xxxyyy/various
    // Length: 1 byte: opcode
    LdR8FromR8 { dst: R8, src: R8 },


    // LD r, n: Load register (immediate)
    // Description: Load to the 8-bit register r, the immediate data n
    // Opcode: 0b00xxx110/various
    // Length: 2 bytes: opcode + n
    LdR8FromImm8 { dst: R8, imm: u8 },


    // LD r, HL: Load register (indirect HL)
    // Description: Load to the 8-bit register r, the data from the absolute address specified by the 16-bit HL register
    // Opcode: 0b01xxx110/various
    // Length: 1 byte: opcode
    LdR8FromHl { dst: R8 },


    // LD HL, r: Load from register (indirect HL)
    // Description: Load to the absolute address specified by the 16-bit HL register, the data from the 8-bit register r
    // Opcode: 0b01110xxx/various
    // Length: 1 byte: opcode
    LdHlFromR8 { src: R8 },


    // LD HL, n: Load from immediate data (indirect HL)
    // Description: Load to the absolute address specified by the 16-bit HL register, the immediate data n
    // Opcode: 0b00110110/0x36
    // Length: 2 bytes: opcode + n
    LdHlFromImm8 { imm: u8 },


    // LD A, BC: Load accumulator (indirect BC)
    // Description: Load to the 8-bit A register, the data from the absolute address specified by the 16-bit BC register
    // Opcode: 0b00001010/0x0A
    // Length: 1 byte: opcode
    LdAFromBc,


    // LD A, DE: Load accumulator (indirect DE)
    // Description: Load to the 8-bit A register, the data from the absolute address specified by the 16-bit DE register
    // Opcode: 0b00011010/0x1A
    // Length: 1 byte: opcode
    LdAFromDe,


    // LD BC, A: Load from accumulator (indirect BC)
    // Description: Load to the absolute address specified by the 16-bit BC register, the data from the 8-bit A register
    // Opcode: 0b00000010/0x02
    // Length: 1 byte: opcode
    LdBcFromA,


    // LD DE, A: Load from accumulator (indirect DE)
    // Description: Load to the absolute address specified by the 16-bit DE register, the data from the 8-bit A register
    // Opcode: 0b00010010/0x12
    // Length: 1 byte: opcode
    LdDeFromA,


    // LD A, nn: Load accumulator (direct)
    // Description: Load to the 8-bit A register, the data from the absolute address specified by the 16-bit operand nn
    // Opcode: 0b11111010/0xFA
    // Length: 3 bytes: opcode + LSB(nn) + MSB(nn)
    LdAFromImm16 { imm: u16 },


    // LD nn, A: Load from accumulator (direct)
    // Description: Load to the absolute address specified by the 16-bit operand nn, data from the 8-bit A register
    // Opcode: 0b11101010/0xEA
    // Length: 3 bytes: opcode + LSB(nn) + MSB(nn)
    LdImm16FromA { imm: u16 },


    // LDH A, C: Load accumulator (indirect 0xFF00 + C)
    // Description: Load to the 8-bit A register, data from the address specified by the 8-bit C register. The full 16-bit absolute address is obtained by setting the most significant byte to 0xFF and the least significant byte to the value of C, so the possible range is 0xFF00 to 0xFFFF
    // Opcode: 0b11110010/0xF2
    // Length: 1 byte: opcode
    LdhAFromC,


    // LDH C, A: Load from accumulator (indirect 0xFF00 + C)
    // Description: Load to the address specified by the 8-bit C register, data from the 8-bit A register. The full 16-bit absolute address is obtained by setting the most significant byte to 0xFF and the least significant byte to the value of C, so the possible range is 0xFF00 to 0xFFFF
    // Opcode: 0b11100010/0xE2
    // Length: 1 byte: opcode
    LdhCFromA,


    // LDH A, n: Load accumulator (direct 0xFF00 + n)
    // Description: Load to the 8-bit A register, data from the address specified by the 8-bit immediate data n. The full 16-bit absolute address is obtained by setting the most significant byte to 0xFF and the least significant byte to the value of n, so the possible range is 0xFF00 to 0xFFFF
    // Opcode: 0b11110000/0xF0
    // Length: 2 bytes: opcode + n
    LdhAFromImm8 { imm: u8 },


    // LDH n, A: Load from accumulator (direct 0xFF00 + n)
    // Description: Load to the address specified by the 8-bit immediate data n, data from 8-bit A register. The full 16-bit absolute address is obtained by setting the most significant byte to 0xFF and the least significant byte to the value of n, so the possible range is 0xFF00 to 0xFFFF
    // Opcode: 0b11100000/0xE0
    // Length: 2 bytes: opcode + n
    LdhImm8FromA { imm: u8 },


    // LD A, HL-: Load accumulator (indirect HL, decrement)
    // Description: Load to the 8-bit A register, data from the absolute address specified by the 16-bit register HL. The value of HL is decremented after the memory read
    // Opcode: 0b00111010/0x3A
    // Length: 1 byte: opcode
    LdAFromHlDec,


    // LD HL-, A: Load from accumulator (indirect HL, decrement)
    // Description: Load to the absolute address specified by the 16-bit register HL, data from the 8-bit A register. The value of HL is decremented after the memory write
    // Opcode: 0b00110010/0x32
    // Length: 1 byte: opcode
    LdHlDecFromA,


    // LD A, HL+: Load accumulator (indirect HL, increment)
    // Description: Load to the 8-bit A register, data from the absolute address specified by the 16-bit register HL. The value of HL is incremented after the memory read
    // Opcode: 0b00101010/0x2A
    // Length: 1 byte: opcode
    LdAFromHlInc,


    // LD HL+, A: Load from accumulator (indirect HL, increment)
    // Description: Load to the absolute address specified by the 16-bit register HL, data from the 8-bit A register. The value of HL is incremented after the memory write
    // Opcode: 0b00100010/0x22
    // Length: 1 byte: opcode
    LdHlIncFromA,


    // --- 8-BIT LOAD INSTRUCTIONS END ---


    // --- 16-BIT LOAD INSTRUCTIONS BEGIN ---


    // LD rr, nn: Load 16-bit register / register pair
    // Description: Load to the 16-bit register rr, the immediate 16-bit data nn
    // Opcode: 0b00xx0001/various
    // Length: 3 bytes: opcode + LSB(nn) + MSB(nn)
    LdR16FromImm16 { dst: R16, imm: u16 },


    // LD nn, SP: Load from stack pointer (direct)
    // Description: Load to the absolute address specified by the 16-bit operand nn, data from the 16-bit SP register
    // Opcode: 0b00001000/0x08
    // Length: 3 bytes: opcode + LSB(nn) + MSB(nn)
    LdImm16FromSp { imm: u16 },


    // LD SP, HL: Load stack pointer from HL
    // Description: Load to the 16-bit SP register, data from the 16-bit HL register
    // Opcode: 0b11111001/0xF9
    // Length: 1 byte: opcode
    LdSpFromHl,


    // PUSH rr: Push to stack
    // Description: Push to stack memory, data from the 16-bit register rr
    // Opcode: 0b11xx0101/various
    // Length: 1 byte: opcode
    PushR16 { src: R16 },


    // POP rr: Pop from stack
    // Description: Pops to the 16-bit register rr, data from the stack memory
    // Opcode: 0b11xx0001/various
    // Length: 1 byte: opcode
    PopR16 { dst: R16 },


    // LD HL, SP + e: Load HL from adjusted stack pointer
    // Description: Load to the HL register, 16-bit data calculated by adding the signed 8-bit operand e to the 16-bit value of the SP register
    // Opcode: 0b11111000/0xF8
    // Length: 2 bytes: opcode + e
    LdHlFromAdjSp { imm: i8 },


    // --- 16-BIT LOAD INSTRUCTIONS END ---


    // --- 8-BIT ARITHMETIC AND LOGICAL INSTRUCTIONS BEGIN ---
    // --- 8-BIT ARITHMETIC AND LOGICAL INSTRUCTIONS END ---


    // --- 16-BIT ARITHMETIC AND LOGICAL INSTRUCTIONS BEGIN ---
    // --- 16-BIT ARITHMETIC AND LOGICAL INSTRUCTIONS END ---


    // --- ROTATE, SHIFT, AND BIT OPERATION INSTRUCTIONS BEGIN ---
    // --- ROTATE, SHIFT, AND BIT OPERATION INSTRUCTIONS END ---
    

    // --- CONTROL FLOW INSTRUCTIONS BEGIN ---
    // --- CONTROL FLOW INSTRUCTIONS END ---


    // --- MISCELLANEOUS INSTRUCTIONS BEGIN ---
    // --- MISCELLANEOUS INSTRUCTIONS END ---
}


impl Instruction
{
    fn from_byte(byte: u8) -> Option<Self>
    {
        match byte
        {
            // 0x02 => Some(Instruction::LdReg { dest: , src: }),
            _ => None,
        }
    }
}





#[derive(Debug, Default)]
struct Registers
{
    // Accumulator
    a: u8,

    // General Purpose
    b: u8,

    // General Purpose
    c: u8,

    // General Purpose
    d: u8,

    // General Purpose
    e: u8,

    // Flags
    f: u8,

    // General Purpose
    h: u8,

    // General Purpose
    l: u8,
}


impl Registers
{
    fn get_af(&self) -> u16
    {
        ((self.a as u16) << 8) | (self.f as u16)
    }

    fn set_af(&mut self, val: u16)
    {
        self.a = ((val & 0xFF00) >> 8) as u8;
        self.f = (val & 0xFF) as u8;
    }

    fn get_bc(&self) -> u16
    {
        ((self.b as u16) << 8) | (self.c as u16)
    }

    fn set_bc(&mut self, val: u16)
    {
        self.b = ((val & 0xFF00) >> 8) as u8;
        self.c = (val & 0xFF) as u8;
    }

    fn get_de(&self) -> u16
    {
        ((self.d as u16) << 8) | (self.e as u16)
    }

    fn set_de(&mut self, val: u16)
    {
        self.d = ((val & 0xFF00) >> 8) as u8;
        self.e = (val & 0xFF) as u8;
    }

    fn get_hl(&self) -> u16
    {
        ((self.h as u16) << 8) | (self.l as u16)
    }

    fn set_hl(&mut self, val: u16)
    {
        self.h = ((val & 0xFF00) >> 8) as u8;
        self.l = (val & 0xFF) as u8;
    }
}


#[derive(Debug, Default)]
pub struct GbCpu
{
    // 8, 8-bit registers which can be combined into 4, 16-bit registers if needed
    registers: Registers,

    // Stack Pointer
    sp: u16,

    // Program Counter
    pc: u16,
}


impl GbCpu
{
    pub fn step(&mut self)
    {
        // Performs the fetch, decode, and execute cycle

    }

    fn execute(&mut self, instr: Instruction)
    {
        // The instruction should already contain both the opcode and any operands
        // And then at this point we can do some kind of function dispatch to execute the instruction
        match instr
        {
            _ => ()
        }
    }
}












// --- UNIT TESTS BEGIN ---

#[cfg(test)]
mod tests
{
    use super::*;


    mod registers_tests
    {
        use super::*;


        #[test]
        fn test_get_af()
        {
            let registers = Registers {
                a: 0x4F,
                f: 0xD9,
                ..Default::default()
            };

            let result = registers.get_af();
            assert_eq!(result, 0x4FD9);
        }

        #[test]
        fn test_set_af()
        {
            let mut registers = Registers::default();

            registers.set_af(0x4FD9);
            assert_eq!(registers.get_af(), 0x4FD9);
        }

        #[test]
        fn test_get_bc()
        {
            let registers = Registers {
                b: 0x72,
                c: 0xFF,
                ..Default::default()
            };

            let result = registers.get_bc();
            assert_eq!(result, 0x72FF);
        }

        #[test]
        fn test_set_bc()
        {
            let mut registers = Registers::default();

            registers.set_bc(0x72FF);
            assert_eq!(registers.get_bc(), 0x72FF);
        }

        #[test]
        fn test_get_de()
        {
            let registers = Registers {
                d: 0x28,
                e: 0x92,
                ..Default::default()
            };

            let result = registers.get_de();
            assert_eq!(result, 0x2892);
        }

        #[test]
        fn test_set_de()
        {
            let mut registers = Registers::default();

            registers.set_de(0x2892);
            assert_eq!(registers.get_de(), 0x2892);
        }

        #[test]
        fn test_get_hl()
        {
            let registers = Registers {
                h: 0xE9,
                l: 0xA7,
                ..Default::default()
            };

            let result = registers.get_hl();
            assert_eq!(result, 0xE9A7);
        }

        #[test]
        fn test_set_hl()
        {
            let mut registers = Registers::default();

            registers.set_hl(0xE9A7);
            assert_eq!(registers.get_hl(), 0xE9A7);
        }
    }


    mod gb_cpu_tests
    {
        use super::*;


        #[test]
        fn test()
        {

        }
    }

}


// --- UNIT TESTS END ---


