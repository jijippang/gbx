
use super::gb_mmu::{MmuReadError, GbMmu, Data, Address};
use tracing::{info, warn};




const MASTER_CLK_FREQ: f64 = 4.194304e6;





#[derive(Debug, PartialEq)]
enum R8
{
    // 0b111
    A, 

    // 0b000
    B, 

    // 0b001
    C, 

    // 0b010
    D, 

    // 0b011
    E, 

    // 0b100
    H, 

    // 0b101
    L,
}


impl R8
{
    fn from_byte(byte: u8) -> Result<Self, String>
    {
        match byte
        {
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
enum R16
{
    // 0b00
    BC, 

    // 0b01
    DE, 

    // 0b10
    HL,
}


impl R16
{
    fn from_byte(byte: u8) -> Result<Self, String>
    {
        match byte
        {
            0b00 => Ok(Self::BC),
            0b01 => Ok(Self::DE),
            0b10 => Ok(Self::HL),
            _ => Err(format!("Unknown byte: {:#010b}, cannot decode R16", byte)),
        }
    }
}


#[derive(Debug, PartialEq)]
enum BitIdx
{
    B0, B1, B2, B3, B4, B5, B6, B7,
}


#[derive(Debug, PartialEq)]
enum Cond
{
    Z, NZ, C, NC,
}


#[derive(Debug, PartialEq)]
#[repr(u8)]
enum RstVec
{
    V0 = 0x00, V1 = 0x08, V2 = 0x10, V3 = 0x18,
    V4 = 0x20, V5 = 0x28, V6 = 0x30, V7 = 0x38,
}


#[derive(Debug, PartialEq)]
#[repr(u8)]
enum IntVec
{
    V0 = 0x40, V1 = 0x48, V2 = 0x50, V3 = 0x58, V4 = 0x60,
}


#[derive(Debug, PartialEq)]
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


    // ADD r: Add (register)
    // Description: Adds to the 8-bit A register, the 8-bit register r, and stores the result back into the A register
    // Opcode: 0b10000xxx/various
    // Length: 1 byte: opcode
    AddR8ToA { src: R8 },


    // ADD HL: Add (indirect HL)
    // Description: Adds to the 8-bit A register, data from the absolute address specified by the 16-bit register HL, and stores the result back into the A register
    // Opcode: 0b10000110/0x86
    // Length: 1 byte: opcode
    AddHlToA,


    // ADD n: Add (immediate)
    // Description: Adds to the 8-bit A register, the immediate data n, and stores the result back into the A register
    // Opcode: 0b11000110/0xC6
    // Length: 2 bytes: opcode + n
    AddImm8ToA { imm: u8 },


    // ADC r: Add with carry (register)
    // Description: Adds to the 8-bit A register, the carry flag and the 8-bit register r, and stores the result back into the A register
    // Opcode: 0b10001xxx/various
    // Length: 1 byte: opcode
    AdcR8ToA { src: R8 },


    // ADC HL: Add with carry (indirect HL)
    // Description: Adds to the 8-bit A register, the carry flag and data from the absolute address specified by the 16-bit register HL, and stores the result back into the A register
    // Opcode: 0b10001110/0x8E
    // Length: 1 byte: opcode
    AdcHlToA,


    // ADC n: Add with carry (immediate)
    // Description: Adds to the 8-bit A register, the carry flag and the immediate data n, and stores the result back into the A register
    // Opcode: 0b11001110/0xCE
    // Length: 2 bytes: opcode + n
    AdcImm8ToA { imm: u8 },


    // SUB r: Subtract (register)
    // Description: Subtracts from the 8-bit A register, the 8-bit register r, and stores the result back into the A register
    // Opcode: 0b10010xxx/various
    // Length: 1 byte: opcode
    SubR8FromA { src: R8 },


    // SUB HL: Subtract (indirect HL)
    // Description: Subtracts from the 8-bit A register, data from the absolute address specified by the 16-bit register HL, and stores the result back into the A register
    // Opcode: 0b10010110/0x96
    // Length: 1 byte: opcode
    SubHlFromA,


    // SUB n: Subtract (immediate)
    // Description: Subtracts from the 8-bit A register, the immediate data n, and stores the result back into the A register
    // Opcode: 0b11010110/0xD6
    // Length: 2 bytes: opcode + n
    SubImm8FromA { imm: u8 },


    // SBC r: Subtract with carry (register)
    // Description: Subtracts from the 8-bit A register, the carry flag and the 8-bit register r, and stores the result back into the A register
    // Opcode: 0b10011xxx/various
    // Length: 1 byte: opcode
    SbcR8FromA { src: R8 },


    // SBC HL: Subtract with carry (indirect HL)
    // Description: Subtracts from the 8-bit A register, the carry flag and data from the absolute address specified by the 16-bit register HL, and stores the result back into the A register
    // Opcode: 0b10011110/0x9E
    // Length: 1 byte: opcode
    SbcHlFromA,


    // SBC n: Subtract with carry (immediate)
    // Description: Subtracts from the 8-bit A register, the carry flag and the immediate data n, and stores the result back into the A register
    // Opcode: 0b11011110/0xDE
    // Length: 2 bytes: opcode + n
    SbcImm8FromA { imm: u8 },


    // CP r: Compare (register)
    // Description: Subtracts from the 8-bit A register, the 8-bit register r, and updates flags based on the result. This instruction is basically identical to SUB r, but does not update the A register
    // Opcode: 0b10111xxx/various
    // Length: 1 byte: opcode
    CpR8WithA { src: R8 },


    // CP HL: Compare (indirect HL)
    // Description: Subtracts from the 8-bit A register, data from the absolute address specified by the 16-bit register HL, and updates flags based on the result. This instruction is basically identical to SUB HL, but does not update the A register
    // Opcode: 0b10111110/0xBE
    // Length: 1 byte: opcode
    CpHlWithA,


    // CP n: Compare (immediate)
    // Description: Subtracts from the 8-bit A register, the immediate data n, and updates flags based on the result. This instruction is basically identical to SUB n, but does not update the A register
    // Opcode: 0b11111110/0xFE
    // Length: 2 bytes: opcode + n
    CpImm8WithA { imm: u8 },


    // INC r: Increment (register)
    // Description: Increments data in the 8-bit register r
    // Opcode: 0b00xxx100/various
    // Length: 1 byte: opcode
    IncR8 { src: R8 },


    // INC HL: Increment (indirect HL)
    // Description: Increments data at the absolute address specified by the 16-bit register HL
    // Opcode: 0b00110100/0x34
    // Length: 1 byte: opcode
    IncHl,


    // DEC r: Decrement (register)
    // Description: Decrements data in the 8-bit register r
    // Opcode: 0b00xxx101/various
    // Length: 1 byte: opcode
    DecR8 { src: R8 },


    // DEC HL: Decrement (indirect HL)
    // Description: Decrements data at the absolute address specified by the 16-bit register HL
    // Opcode: 0b00110101/0x35
    // Length: 1 byte: opcode
    DecHl,


    // AND r: Bitwise AND (register)
    // Description: Performs a bitwise AND operation between the 8-bit A register and the 8-bit register r, and stores the result back into the A register
    // Opcode: 0b10100xxx/various
    // Length: 1 byte: opcode
    AndAWithR8 { src: R8 },


    // AND HL: Bitwise AND (indirect HL)
    // Description: Performs a bitwise AND operation between the 8-bit A register and the data from the absolute address specified by the 16-bit register HL, and stores the result back into the A register
    // Opcode: 0b10100110/0xA6
    // Length: 1 byte: opcode
    AndAWithHl,


    // AND n: Bitwise AND (immediate)
    // Description: Performs a bitwise AND operation between the 8-bit A register and immediate data n, and stores the result back into the A register
    // Opcode: 0b11100110/0xE6
    // Length: 2 bytes: opcode + n
    AndAWithImm8 { imm: u8 },


    // OR r: Bitwise OR (register)
    // Description: Performs a bitwise OR operation between the 8-bit A register and the 8-bit register r, and stores the result back into the A register
    // Opcode: 0b10110xxx/various
    // Length: 1 byte: opcode
    OrAWithR8 { src: R8 },


    // OR HL: Bitwise OR (indirect HL)
    // Description: Performs a bitwise OR operation between the 8-bit A register and the data from the absolute address specified by the 16-bit register HL, and stores the result back into the A register
    // Opcode: 0b10110110/0xB6
    // Length: 1 byte: opcode
    OrAWithHl,


    // OR n: Bitwise OR (immediate)
    // Description: Performs a bitwise OR operation between the 8-bit A register and immediate data n, and stores the result back into the A register
    // Opcode: 0b11110110/0xF6
    // Length: 2 bytes: opcode + n
    OrAWithImm8 { imm: u8 },


    // XOR r: Bitwise XOR (register)
    // Description: Performs a bitwise XOR operation between the 8-bit A register and the 8-bit register r, and stores the result back into the A register
    // Opcode: 0b10101xxx/various
    // Length: 1 byte: opcode
    XorAWithR8 { src: R8 },


    // XOR HL: Bitwise XOR (indirect HL)
    // Description: Performs a bitwise XOR operation between the 8-bit A register and the data from the absolute address specified by the 16-bit register HL, and stores the result back into the A register
    // Opcode: 0b10101110/0xAE
    // Length: 1 byte: opcode
    XorAWithHl,


    // XOR n: Bitwise XOR (immediate)
    // Description: Performs a bitwise XOR operation between the 8-bit A register and immediate data n, and stores the result back into the A register
    // Opcode: 0b11101110/0xEE
    // Length: 2 bytes: opcode + n
    XorAWithImm8 { imm: u8 },


    // CCF: Complement carry flag
    // Description: Flips the carry flag, and clears the N and H flags
    // Opcode: 0b00111111/0x3F
    // Length: 1 byte: opcode
    Ccf,


    // SCF: Set carry flag
    // Description: Sets the carry flag, and clears the N and H flags
    // Opcode: 0b00110111/0x37
    // Length: 1 byte: opcode
    Scf,


    // DAA: Decimal adjust accumulator
    // Description: Behavior depends on the N flag,
    //
    // If the N flag is set:
    // 1. Initialize the 8-bit adjustment value to 0
    // 2. If the H flag is set, then add 0x06 to the adjustment
    // 3. If the C flag is set, then add 0x60 to the adjustment
    // 4. Subtract the adjustment from the 8-bit A register
    //
    // If the N flag is not set:
    // 1. Initialize the 8-bit adjustment value to 0
    // 2. If the H flag is set or A & 0x0F > 0x09, then add 0x06 to the adjustment
    // 3. If the C flag is set or A > 0x99, then add 0x60 to the adjustment and set the C flag
    // 4. Add the adjustment to the 8-bit A register
    //
    // Opcode: 0b00100111/0x27
    // Length: 1 byte: opcode
    Daa,


    // CPL: Complement accumulator
    // Description: Flips all the bits in the 8-bit A register, and sets the N and H flags
    // Opcode: 0b00101111/0x2F
    // Length: 1 byte: opcode
    Cpl,


    // --- 8-BIT ARITHMETIC AND LOGICAL INSTRUCTIONS END ---


    // --- 16-BIT ARITHMETIC INSTRUCTIONS BEGIN ---


    // INC rr: Increment 16-bit register
    // Description: Increments data in the 16-bit register rr
    // Opcode: 0b00xx0011/various
    // Length: 1 byte: opcode
    IncR16 { src: R16 },


    // DEC rr: Decrement 16-bit register
    // Description: Decrements data in the 16-bit register rr
    // Opcode: 0b00xx1011/various
    // Length: 1 byte: opcode
    DecR16 { src: R16 },


    // ADD HL, rr: Add (16-bit register)
    // Description: Adds to the 16-bit HL register pair, the 16-bit register rr, and stores the result back into the HL register pair
    // Opcode: 0b00xx1001/various
    // Length: 1 byte: opcode
    AddR16ToHl { src: R16 },


    // ADD SP, e: Add to stack pointer (relative)
    // Description: Adds to the 16-bit SP register, 16-bit data calculated by adding the signed 8-bit operand e to the 16-bit value of the SP register
    // Opcode: 0b11101000/0xE8
    // Length: 2 bytes: opcode + e
    AddAdjSpToSp { imm: i8 },


    // --- 16-BIT ARITHMETIC INSTRUCTIONS END ---


    // --- ROTATE, SHIFT, AND BIT OPERATION INSTRUCTIONS BEGIN ---


    // RLCA: Rotate left circular (accumulator)
    // Description: Rotates the 8-bit A register left in a circular manner (carry flag is updated but not used)
    //
    // Every bit is shifted to the left (e.g. bit 1 value is copied from bit 0). Bit 7 is copied both to bit 0 and the carry flag
    // Note that unlike the related RLC r instruction, RLCA always sets the zero flag to 0 without looking at the resulting value of the calculation
    //
    // Opcode: 0b00000111/0x07
    // Length: 1 byte: opcode
    RlcA,


    // RRCA: Rotate right circular (accumulator)
    // Description: Rotates the 8-bit A register right in a circular manner (carry flag is updated but not used)
    //
    // Every bit is shifted to the right (e.g. bit 1 value is copied from bit 0). Bit 0 is copied both to bit 7 and the carry flag
    // Note that unlike the related RRC r instruction, RRCA always sets the zero flag to 0 without looking at the resulting value of the calculation
    //
    // Opcode: 0b00001111/0x0F
    // Length: 1 byte: opcode
    RrcA,


    // RLA: Rotate left (accumulator)
    // Description: Rotates the 8-bit A register left through the carry flag
    //
    // Every bit is shifted to the left (e.g. bit 1 value is copied from bit 0). The carry flag is copied to bit 0, and bit 7 is copied to the carry flag
    // Note that unlike the related RL r instruction, RLA always sets the zero flag to 0 without looking at the resulting value of the calculation
    //
    // Opcode: 0b00010111/0x17
    // Length: 1 byte: opcode
    RlA,


    // RRA: Rotate right (accumulator)
    // Description: Rotates the 8-bit A register right through the carry flag
    //
    // Every bit is shifted to the right (e.g. bit 1 value is copied to bit 0). The carry flag is copied to bit 7, and bit 0 is copied to the carry flag
    // Note that unlike the related RR r instruction, RRA always sets the zero flag to 0 without looking at the resulting value of the calculation
    //
    // Opcode: 0b00011111/0x1F
    // Length: 1 byte: opcode
    RrA,


    // RLC r: Rotate left circular (register)
    // Description: Rotates the 8-bit register r value left in a circular manner (carry flag is updated but not used)
    //
    // Every bit is shifted to the left (e.g. bit 1 value is copied from bit 0). Bit 7 is copied both to bit 0 and the carry flag
    //
    // Opcode: 0b00000xxx/various
    // Length: 2 bytes: CB prefix + opcode
    RlcR8 { src: R8 },


    // RLC HL: Rotate left circular (indirect HL)
    // Description: Rotates the 8-bit data at the absolute address specified by the 16-bit register HL, left in a circular manner (carry flag is updated but not used)
    //
    // Every bit is shifted to the left (e.g. bit 1 value is copied from bit 0). Bit 7 is copied both to bit 0 and the carry flag
    //
    // Opcode: 0b00000110/0x06
    // Length: 2 bytes: CB prefix + opcode
    RlcHl,


    // RRC r: Rotate right circular (register)
    // Description: Rotates the 8-bit register r value right in a circular manner (carry flag is updated but not used)
    //
    // Every bit is shifted to the right (e.g. bit 1 value is copied to bit 0). Bit 0 is copied both to bit 7 and the carry flag
    //
    // Opcode: 0b00001xxx/various
    // Length: 2 bytes: CB prefix + opcode
    RrcR8 { src: R8 },


    // RRC HL: Rotate right circular (indirect HL)
    // Description: Rotates the 8-bit data at the absolute address specified by the 16-bit register HL, right in a circular manner (carry flag is updated but not used)
    //
    // Every bit is shifted to the right (e.g. bit 1 value is copied to bit 0). Bit 0 is copied both to bit 7 and the carry flag
    //
    // Opcode: 0b00001110/0x0E
    // Length: 2 bytes: CB prefix + opcode
    RrcHl,


    // RL r: Rotate left (register)
    // Description: Rotates the 8-bit register r value left through the carry flag
    //
    // Every bit is shifted to the left (e.g. bit 1 value is copied from bit 0). The carry flag is copied to bit 0, and bit 7 is copied to the carry flag
    //
    // Opcode: 0b00010xxx/various
    // Length: 2 bytes: CB prefix + opcode
    RlR8 { src: R8 },


    // RL HL: Rotate left (indirect HL)
    // Description: Rotates the 8-bit data at the absolute address specified by the 16-bit register HL, left through the carry flag
    //
    // Every bit is shifted to the left (e.g. bit 1 value is copied from bit 0). The carry flag is copied to bit 0, and bit 7 is copied to the carry flag
    //
    // Opcode: 0b00010110/0x16
    // Length: 2 bytes: CB prefix + opcode
    RlHl,


    // RR r: Rotate right (register)
    // Description: Rotates the 8-bit register r value right through the carry flag
    //
    // Every bit is shifted to the right (e.g. bit 1 value is copied to bit 0). The carry flag is copied to bit 7, and bit 0 is copied to the carry flag
    //
    // Opcode: 0b00011xxx/various
    // Length: 2 bytes: CB prefix + opcode
    RrR8 { src: R8 },


    // RR HL: Rotate right (indirect HL)
    // Description: Rotates the 8-bit data at the absolute address specified by the 16-bit register HL, right through the carry flag
    //
    // Every bit is shifted to the right (e.g. bit 1 value is copied to bit 0). The carry flag is copied to bit 7, and bit 0 is copied to the carry flag
    //
    // Opcode: 0b00011110/0x1E
    // Length: 2 bytes: CB prefix + opcode
    RrHl,


    // SLA r: Shift left arithmetic (register)
    // Description: Shifts the 8-bit register r value left by one bit using an arithmetic shift
    //
    // Bit 7 is shifted to the carry flag, and bit 0 is set to a fixed value of 0
    //
    // Opcode: 0b00100xxx/various
    // Length: 2 bytes: CB prefix + opcode
    SlaR8 { src: R8 },


    // SLA HL: Shift left arithmetic (indirect HL)
    // Description: Shifts the 8-bit value at the address specified by the HL register, left by one bit using an arithmetic shift
    //
    // Bit 7 is shifted to the carry flag, and bit 0 is set to a fixed value of 0
    //
    // Opcode: 0b00100110/0x26
    // Length: 2 bytes: CB prefix + opcode
    SlaHl,


    // SRA r: Shift right arithmetic (register)
    // Description: Shifts the 8-bit register r value right by one bit using an arithmetic shift
    //
    // Bit 7 retains its value, and bit 0 is shifted to the carry flag
    //
    // Opcode: 0b00101xxx/various
    // Length: 2 bytes: CB prefix + opcode
    SraR8 { src: R8 },


    // SRA HL: Shift right arithmetic (indirect HL)
    // Description: Shifts the 8-bit value at the address specified by the HL register, right by one bit using an arithmetic shift
    //
    // Bit 7 retains its value, and bit 0 is shifted to the carry flag
    //
    // Opcode: 0b00101110/0x2E
    // Length: 2 bytes: CB prefix + opcode
    SraHl,


    // SWAP r: Swap nibbles (register)
    // Description: Swaps the high and low 4-bit nibbles of the 8-bit register r
    // Opcode: 0b00110xxx/various
    // Length: 2 bytes: CB prefix + opcode
    SwapR8 { src: R8 },


    // SWAP HL: Swap nibbles (indirect HL)
    // Description: Swaps the high and low 4-bit nibbles of the 8-bit data at the absolute address specified by the 16-bit register HL
    // Opcode: 0b00110110/0x36
    // Length: 2 bytes: CB prefix + opcode
    SwapHl,


    // SRL r: Shift right logical (register)
    // Description: Shifts the 8-bit register r value right by one bit using a logical shift
    //
    // Bit 7 is set to a fixed value of 0, and bit 0 is shifted to the carry flag
    //
    // Opcode: 0b00111xxx/various
    // Length: 2 bytes: CB prefix + opcode
    SrlR8 { src: R8 },


    // SRL HL: Shift right logical (indirect HL)
    // Description: Shifts the 8-bit value at the address specified by the HL register, right by one bit using a logical shift
    //
    // Bit 7 is set to a fixed value of 0, and bit 0 is shifted to the carry flag
    //
    // Opcode: 0b00111110/0x3E
    // Length: 2 bytes: CB prefix + opcode
    SrlHl,


    // BIT b, r: Test bit (register)
    // Description: Tests the bit b of the 8-bit register r
    //
    // The zero flag is set to 1 if the chosen bit is 0, and 0 otherwise
    //
    // Opcode: 0b01xxxyyy/various
    // Length: 2 bytes: CB prefix + opcode
    TstBitOfR8 { bit: BitIdx, src: R8 },


    // BIT b, HL: Test bit (indirect HL)
    // Description: Tests the bit b of the 8-bit data at the absolute address specified by the 16-bit register HL
    //
    // The zero flag is set to 1 if the chosen bit is 0, and 0 otherwise
    //
    // Opcode: 0b01xxx110/various
    // Length: 2 bytes: CB prefix + opcode
    TstBitOfHl { bit: BitIdx },


    // RES b, r: Reset bit (register)
    // Description: Resets the bit b of the 8-bit register r to 0
    // Opcode: 0b10xxxyyy/various
    // Length: 2 bytes: CB prefix + opcode
    ResBitOfR8 { bit: BitIdx, src: R8 },


    // RES b, HL: Reset bit (indirect HL)
    // Description: Resets the bit b of the 8-bit data at the absolute address specified by the 16-bit register HL, to 0
    // Opcode: 0b10xxx110/various
    // Length: 2 bytes: CB prefix + opcode
    ResBitOfHl { bit: BitIdx },


    // SET b, r: Set bit (register)
    // Description: Sets the bit b of the 8-bit register r to 1
    // Opcode: 0b11xxxyyy/various
    // Length: 2 bytes: CB prefix + opcode
    SetBitOfR8 { bit: BitIdx, src: R8 },


    // SET b, HL: Set bit (indirect HL)
    // Description: Sets the bit b of the 8-bit data at the absolute address specified by the 16-bit register HL, to 1
    // Opcode: 0b11xxx110/various
    // Length: 2 bytes: CB prefix + opcode
    SetBitOfHl { bit: BitIdx },


    // --- ROTATE, SHIFT, AND BIT OPERATION INSTRUCTIONS END ---
    

    // --- CONTROL FLOW INSTRUCTIONS BEGIN ---


    // JP nn: Jump
    // Description: Unconditional jump to the absolute address specified by the 16-bit immediate operand nn
    // Opcode: 0b11000011/0xC3
    // Length: 3 bytes: opcode + LSB(nn) + MSB(nn)
    JpImm16 { imm: u16 },


    // JP HL: Jump to HL
    // Description: Unconditional jump to the absolute address specified by the 16-bit register HL
    // Opcode: 0b11101001/0xE9
    // Length: 1 byte: opcode
    JpHl,


    // JP cc, nn: Jump (conditional)
    // Description: Conditional jump to the absolute address specified by the 16-bit operand nn, depending on the condition cc
    //
    // Note that the operand (absolute address) is read even when the condition is false!
    //
    // Opcode: 0b110xx010/various
    // Length: 3 bytes: opcode + LSB(nn) + MSB(nn)
    JpCondImm16 { cond: Cond, imm: u16 },


    // JR e: Relative jump
    // Description: Unconditional jump to the relative address specified by the signed 8-bit operand e
    // Opcode: 0b00011000/0x18
    // Length: 2 bytes: opcode + e
    JrImm8 { imm: i8 },


    // JR cc, e: Relative jump (conditional)
    // Description: Conditional jump to the relative address specified by the signed 8-bit operand e, depending on the condition cc
    //
    // Note that the operand (relative address offset) is read even when the condition is false!
    //
    // Opcode: 0b001xx000/various
    // Length: 2 bytes: opcode + e
    JrCondImm8 { cond: Cond, imm: i8 },


    // CALL nn: Call function
    // Description: Unconditional function call to the absolute address specified by the 16-bit operand nn
    // Opcode: 0b11001101/0xCD
    // Length: 3 bytes: opcode + LSB(nn) + MSB(nn)
    CallImm16 { imm: u16 },


    // CALL cc, nn: Call function (conditional)
    // Description: Conditional function call to the absolute address specified by the 16-bit operand nn, depending on the condition cc
    //
    // Note that the operand (absolute address) is read even when the condition is false!
    //
    // Opcode: 0b110xx100/various
    // Length: 3 bytes: opcode + LSB(nn) + MSB(nn)
    CallCondImm16 { cond: Cond, imm: u16 },


    // RET: Return from function
    // Description: Unconditional return from a function
    // Opcode: 0b11001001/0xC9
    // Length: 1 byte: opcode
    Ret,


    // RET cc: Return from function (conditional)
    // Description: Conditional return from a function, depending on the condition cc
    // Opcode: 0b110xx000/various
    // Length: 1 byte: opcode
    RetCond { cond: Cond },


    // RETI: Return from interrupt handler
    // Description: Unconditional return from a function. Also enables interrupts by setting IME = 1
    // Opcode: 0b11011001/0xD9
    // Length: 1 byte: opcode
    Reti,


    // RST n: Restart / Call function (implied)
    // Description: Unconditional function call to the absolute fixed address defined by the opcode
    // Opcode: 0b11xxx111/various
    // Length: 1 byte: opcode
    RstVec { vec: RstVec },


    // --- CONTROL FLOW INSTRUCTIONS END ---


    // --- MISCELLANEOUS INSTRUCTIONS BEGIN ---
    

    // HALT: Halt system clock
    // Description: Enter CPU low-power consumption mode until an interrupt occurs, behavior depends on the state of the IME flag and whether interrupts are pending (i.e. if IE & IF != 0)
    //
    // If the IME flag is set:
    // The CPU enters low-power mode until after an interrupt is about to be serviced. The handler
    // is executed normally, and the CPU resumes execution after the HALT when that returns
    //
    // If the IME flag is not set, and no interrupts are pending:
    // As soon as an interrupt becomes pending, the CPU resumes execution. This is like the above,
    // except that the handler is not called
    //
    // If the IME flag is not set, and some interrupt is pending:
    // The CPU continues execution after the HALT, but the byte after is read twice in a row (PC is
    // not incremented, due to a hardware bug)
    //
    // Opcode: 0b01110110/0x76
    // Length: 1 byte: opcode
    Halt,


    // NOTE: Implemented as an NOP
    // STOP n: Stop system and main clocks
    // Description: Enter CPU very low-power consumption mode until an interrupt occurs
    // Opcode: 0b00010000/0x10
    // Length: 2 bytes: opcode + n
    Stop,


    // DI: Disable interrupts
    // Description: Disables interrupt handling by setting IME = 0 and cancelling any scheduled effects of the EI instruction if any
    // Opcode: 0b11110011/0xF3
    // Length: 1 byte: opcode
    Di,


    // EI: Enable interrupts
    // Description: Schedules interrupt handling to be enabled after the next machine cycle
    // Opcode: 0b11111011/0xFB
    // Length: 1 byte: opcode
    Ei,


    // NOP: No operation
    // Description: No operation. This instruction doesn't do anything, but can be used to add a delay of one machine cycle and increment the PC by one
    // Opcode: 0b00000000/0x00
    // Length: 1 byte: opcode
    Nop,


    // --- MISCELLANEOUS INSTRUCTIONS END ---
}


impl Instruction
{
    fn from_byte(byte: u8) -> Result<Self, String>
    {
        match byte
        {
            0x40..=0x7F => 
            {
                // To decode the dst and src registers, extract out the corresponding 3 bits each from the
                // opcode and make sure they are right-aligned before calling the R8 constructor
                let dst_bits = (byte & 0x38) >> 3;
                let src_bits = byte & 0x07;
                Ok(Self::LdR8FromR8 { dst: R8::from_byte(dst_bits)?, src: R8::from_byte(src_bits)? })
            }


            // 0xC6 => Ok(Self::AddImm8ToA { imm: }),


            _ => Err(format!("Unknown Opcode: {:#X}, cannot decode Instruction", byte)),
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


impl GbCpu
{
    pub fn step(&mut self)
    {
        // Performs the fetch, decode, and execute cycle

    }

    fn fetch(&mut self) -> Data
    {
        let data = self.memory.read(self.pc);
        self.pc += 1;
        data.unwrap_or_else(|err| { err.into() })
    }

    fn execute(&mut self, instr: &Instruction)
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


    mod r8_tests
    {
        use super::*;


        #[test]
        fn test_from_byte()
        {
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


    mod r16_tests
    {
        use super::*;


        #[test]
        fn test_from_byte()
        {
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


    mod instruction_tests
    {
        use super::*;


        #[test]
        fn test_from_byte()
        {
            let byte = 0b11_111_111;
            let instr = Instruction::from_byte(byte);
            assert!(instr.is_err());

            let byte = 0b01_000_000;
            let instr = Instruction::from_byte(byte).unwrap();
            assert_eq!(instr, Instruction::LdR8FromR8 { dst: R8::from_byte(0b000).unwrap(), src: R8::from_byte(0b000).unwrap() });

            let byte = 0b01_010_101;
            let instr = Instruction::from_byte(byte).unwrap();
            assert_eq!(instr, Instruction::LdR8FromR8 { dst: R8::from_byte(0b010).unwrap(), src: R8::from_byte(0b101).unwrap() });

            let byte = 0b01_111_111;
            let instr = Instruction::from_byte(byte).unwrap();
            assert_eq!(instr, Instruction::LdR8FromR8 { dst: R8::from_byte(0b111).unwrap(), src: R8::from_byte(0b111).unwrap() });
        }
    }


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


