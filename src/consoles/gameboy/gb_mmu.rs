
use tracing::warn;



pub type Data = u8;
pub type Address = u16;


const ROM_SIZE: usize = 0x7FFF - 0x0000 + 1;
const VRAM_SIZE: usize = 0x9FFF - 0x8000 + 1;
const EXTRAM_SIZE: usize = 0xBFFF - 0xA000 + 1;
const WRAM_SIZE: usize = 0xDFFF - 0xC000 + 1;
const OAM_SIZE: usize = 0xFE9F - 0xFE00 + 1;
const HRAM_SIZE: usize = 0xFFFE - 0xFF80 + 1;




#[derive(Debug)]
pub struct GbMmu
{
    // [0x0000, 0x7FFF]
    rom: [Data; ROM_SIZE],

    // [0x8000, 0x9FFF]
    vram: [Data; VRAM_SIZE],

    // [0xA000, 0xBFFF]
    extram: [Data; EXTRAM_SIZE],

    // [0xC000, 0xDFFF]
    wram: [Data; WRAM_SIZE],

    // [0xFE00, 0xFE9F]
    oam: [Data; OAM_SIZE],

    // [0xFF80, 0xFFFE]
    hram: [Data; HRAM_SIZE],
}


impl Default for GbMmu
{
    fn default() -> Self
    {
        Self {
            rom: [0; ROM_SIZE],
            vram: [0; VRAM_SIZE],
            extram: [0; EXTRAM_SIZE],
            wram: [0; WRAM_SIZE],
            oam: [0; OAM_SIZE],
            hram: [0; HRAM_SIZE],
        }
    }
}


impl GbMmu
{
    pub fn read(&self, addr: Address) -> Result<Data, ()>
    {
        match addr
        {
            0x0000..=0x7FFF => Ok(self.rom[addr as usize]),
            0x8000..=0x9FFF => Ok(self.vram[(addr - 0x8000) as usize]),
            0xA000..=0xBFFF => Ok(self.extram[(addr - 0xA000) as usize]),
            0xC000..=0xDFFF => Ok(self.wram[(addr - 0xC000) as usize]),
            0xFE00..=0xFE9F => Ok(self.oam[(addr - 0xFE00) as usize]),
            0xFF80..=0xFFFE => Ok(self.hram[(addr - 0xFF80) as usize]),
            _ => { warn!("Invalid Address: {:#X}, cannot read from memory", addr); Err(()) }
        }
    }

    pub fn write(&mut self, addr: Address, data: Data) -> Result<(), ()>
    {
        match addr
        {
            0x0000..=0x7FFF => { self.rom[addr as usize] = data; Ok(()) }
            0x8000..=0x9FFF => { self.vram[(addr - 0x8000) as usize] = data; Ok(()) }
            0xA000..=0xBFFF => { self.extram[(addr - 0xA000) as usize] = data; Ok(()) }
            0xC000..=0xDFFF => { self.wram[(addr - 0xC000) as usize] = data; Ok(()) }
            0xFE00..=0xFE9F => { self.oam[(addr - 0xFE00) as usize] = data; Ok(()) }
            0xFF80..=0xFFFE => { self.hram[(addr - 0xFF80) as usize] = data; Ok(()) }
            _ => { warn!("Invalid Address: {:#X}, cannot write {:#X} to memory", addr, data); Err(()) }
        }
    }
}







// Memory Bank Controllers
trait Mbc
{

}





// --- UNIT TESTS BEGIN ---


#[cfg(test)]
mod tests
{
    use super::*;


    mod gb_mmu_tests
    {
        use super::*;


        #[test]
        fn test_read()
        {
            let gb_mmu = GbMmu::default();
            let addr = 0xFEAB;
            let result = gb_mmu.read(addr);
            assert!(result.is_err());

            let gb_mmu = GbMmu {
                rom: [0xAB; ROM_SIZE],
                ..Default::default()
            };
            let addr = 0x1307;
            let result = gb_mmu.read(addr).unwrap();
            assert_eq!(result, 0xAB);

            let gb_mmu = GbMmu {
                vram: [0x11; VRAM_SIZE],
                ..Default::default()
            };
            let addr = 0x952C;
            let result = gb_mmu.read(addr).unwrap();
            assert_eq!(result, 0x11);
        }

        #[test]
        fn test_write()
        {
            let mut gb_mmu = GbMmu::default();
            let addr = 0xFEAB;
            let data = 0x69;
            let result = gb_mmu.write(addr, data);
            assert!(result.is_err());

            let mut gb_mmu = GbMmu::default();
            let addr = 0x1307;
            let data = 0xAB;
            let result = gb_mmu.write(addr, data);
            assert!(result.is_ok());
            let result = gb_mmu.read(addr).unwrap();
            assert_eq!(result, 0xAB);

            let mut gb_mmu = GbMmu::default();
            let addr = 0x952C;
            let data = 0x11;
            let result = gb_mmu.write(addr, data);
            assert!(result.is_ok());
            let result = gb_mmu.read(addr).unwrap();
            assert_eq!(result, 0x11);
        }
    }
}


// --- UNIT TESTS END ---


