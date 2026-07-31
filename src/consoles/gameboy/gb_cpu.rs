

#[derive(Debug, Default)]
pub struct GbCpu
{
    // 6, 16-bit registers that can be split up into 2, 8-bit registers if needed
    // In order they are...
    // AF -> Accumulator and Flags
    // BC -> General Purpose
    // DE -> General Purpose
    // HL -> General Purpose
    // SP -> Stack Pointer
    // PC -> Program Counter
    registers: [u16; 6],



}


impl GbCpu
{

}




