

const MASTER_CLK: f64 = 4.194304e6;



#[derive(Debug, Default)]
struct GbCpuRegisters
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

    // Stack Pointer
    sp: u16,

    // Program Counter
    pc: u16,
}


impl GbCpuRegisters
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
    // 6, 16-bit registers that can be split up into 2, 8-bit registers if needed
    registers: GbCpuRegisters,

}


impl GbCpu
{

}





// --- UNIT TESTS BEGIN ---

#[cfg(test)]
mod tests
{
    use super::*;


    mod gb_cpu_registers_tests
    {
        use super::*;


        #[test]
        fn test_get_af()
        {
            let gb_cpu_registers = GbCpuRegisters {
                a: 0x4F,
                f: 0xD9,
                ..Default::default()
            };

            let result = gb_cpu_registers.get_af();
            assert_eq!(result, 0x4FD9);
        }

        #[test]
        fn test_set_af()
        {
            let mut gb_cpu_registers = GbCpuRegisters::default();

            gb_cpu_registers.set_af(0x4FD9);
            assert_eq!(gb_cpu_registers.get_af(), 0x4FD9);
        }

        #[test]
        fn test_get_bc()
        {
            let gb_cpu_registers = GbCpuRegisters {
                b: 0x72,
                c: 0xFF,
                ..Default::default()
            };

            let result = gb_cpu_registers.get_bc();
            assert_eq!(result, 0x72FF);
        }

        #[test]
        fn test_set_bc()
        {
            let mut gb_cpu_registers = GbCpuRegisters::default();

            gb_cpu_registers.set_bc(0x72FF);
            assert_eq!(gb_cpu_registers.get_bc(), 0x72FF);
        }

        #[test]
        fn test_get_de()
        {
            let gb_cpu_registers = GbCpuRegisters {
                d: 0x28,
                e: 0x92,
                ..Default::default()
            };

            let result = gb_cpu_registers.get_de();
            assert_eq!(result, 0x2892);
        }

        #[test]
        fn test_set_de()
        {
            let mut gb_cpu_registers = GbCpuRegisters::default();

            gb_cpu_registers.set_de(0x2892);
            assert_eq!(gb_cpu_registers.get_de(), 0x2892);
        }

        #[test]
        fn test_get_hl()
        {
            let gb_cpu_registers = GbCpuRegisters {
                h: 0xE9,
                l: 0xA7,
                ..Default::default()
            };

            let result = gb_cpu_registers.get_hl();
            assert_eq!(result, 0xE9A7);
        }

        #[test]
        fn test_set_hl()
        {
            let mut gb_cpu_registers = GbCpuRegisters::default();

            gb_cpu_registers.set_hl(0xE9A7);
            assert_eq!(gb_cpu_registers.get_hl(), 0xE9A7);
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








