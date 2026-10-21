use super::{GbCartridge, GbPpu};
use crate::common::gameboy::memory::types::{Address, Data};
use std::error::Error;
use std::fmt;
use tracing::warn;

const OPEN_BUS: Data = 0xFF;
const WRAM_SIZE: usize = 0xDFFF - 0xC000 + 1;
const IO_SIZE: usize = 0xFF7F - 0xFF00 + 1;
const HRAM_SIZE: usize = 0xFFFE - 0xFF80 + 1;

#[derive(Debug)]
pub enum MemoryReadError {
    // Attempting to read from memory locations that are unusable or prohibited
    OutOfBounds(Data),

    // Attempting to read from memory locations that are currently locked
    Locked(Data),

    // Attempting to read from memory locations that are write only
    WriteOnly(Data),
}

impl fmt::Display for MemoryReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutOfBounds(data) => write!(
                f,
                "Attempting to read from memory location that is unusable or prohibited, returning default value of: {:#X}",
                data
            ),
            Self::Locked(data) => write!(
                f,
                "Attempting to read from memory location that is currently locked, returning default value of: {:#X}",
                data
            ),
            Self::WriteOnly(data) => write!(
                f,
                "Attempting to read from memory location that is write only, returning default value of: {:#X}",
                data
            ),
        }
    }
}

impl From<MemoryReadError> for Data {
    fn from(err: MemoryReadError) -> Self {
        match err {
            MemoryReadError::OutOfBounds(data) => data,
            MemoryReadError::Locked(data) => data,
            MemoryReadError::WriteOnly(data) => data,
        }
    }
}

impl Error for MemoryReadError {}

#[derive(Debug)]
pub enum MemoryWriteError {
    // Attempting to write to memory locations that are unusable or prohibited
    OutOfBounds,

    // Attempting to write to memory locations that are currently locked
    Locked,

    // Attempting to write to memory locations that are read only
    ReadOnly,
}

impl fmt::Display for MemoryWriteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutOfBounds => write!(
                f,
                "Attempting to write to memory location that is unusable or prohibited"
            ),
            Self::Locked => write!(
                f,
                "Attempting to write to memory location that is currently locked"
            ),
            Self::ReadOnly => write!(
                f,
                "Attempting to write to memory location that is read only"
            ),
        }
    }
}

impl Error for MemoryWriteError {}

#[derive(Debug)]
pub struct GbMmu {
    // [0x0000, 0x7FFF] -> rom
    // Owned by cartridge

    // [0x8000, 0x9FFF] -> vram
    // Owned by ppu

    // [0xA000, 0xBFFF] -> extram
    // Owned by cartridge

    // [0xC000, 0xDFFF]
    wram: [Data; WRAM_SIZE],

    // [0xE000, 0xFDFF] -> eram (mirror of [0xC000, 0xDDFF] which is a subset of wram)

    // [0xFE00, 0xFE9F] -> oam
    // Owned by ppu

    // [0xFF00, 0xFF7F]
    io: [Data; IO_SIZE],

    // [0xFF80, 0xFFFE]
    hram: [Data; HRAM_SIZE],

    // 0xFFFF
    ie: Data,
}

impl GbMmu {
    pub fn read(
        &self,
        addr: Address,
        cartridge: &GbCartridge,
        ppu: &GbPpu,
    ) -> Result<Data, MemoryReadError> {
        match addr {
            0x0000..=0x7FFF => cartridge.read(addr),
            0x8000..=0x9FFF => ppu.read(addr),
            0xA000..=0xBFFF => cartridge.read(addr),
            0xC000..=0xDFFF => Ok(self.wram[(addr - 0xC000) as usize]),
            0xE000..=0xFDFF => Ok(self.wram[(addr - 0x2000 - 0xC000) as usize]),
            0xFE00..=0xFE9F => ppu.read(addr),
            0xFF00..=0xFF7F => Ok(self.io[(addr - 0xFF00) as usize]),
            0xFF80..=0xFFFE => Ok(self.hram[(addr - 0xFF80) as usize]),
            0xFFFF => Ok(self.ie),
            _ => {
                warn!("Invalid Address: {:#X}, cannot read from memory", addr);
                Err(MemoryReadError::OutOfBounds(OPEN_BUS))
            }
        }
    }

    pub fn write(
        &mut self,
        addr: Address,
        data: Data,
        cartridge: &mut GbCartridge,
        ppu: &mut GbPpu,
    ) -> Result<(), MemoryWriteError> {
        match addr {
            0x0000..=0x7FFF => cartridge.write(addr, data),
            0x8000..=0x9FFF => ppu.write(addr, data),
            0xA000..=0xBFFF => cartridge.write(addr, data),
            0xC000..=0xDFFF => {
                self.wram[(addr - 0xC000) as usize] = data;
                Ok(())
            }
            0xE000..=0xFDFF => {
                self.wram[(addr - 0x2000 - 0xC000) as usize] = data;
                Ok(())
            }
            0xFE00..=0xFE9F => ppu.write(addr, data),
            0xFF00..=0xFF7F => {
                self.io[(addr - 0xFF00) as usize] = data;
                Ok(())
            }
            0xFF80..=0xFFFE => {
                self.hram[(addr - 0xFF80) as usize] = data;
                Ok(())
            }
            0xFFFF => {
                self.ie = data;
                Ok(())
            }
            _ => {
                warn!(
                    "Invalid Address: {:#X}, cannot write {:#X} to memory",
                    addr, data
                );
                Err(MemoryWriteError::OutOfBounds)
            }
        }
    }
}

impl Default for GbMmu {
    fn default() -> Self {
        Self {
            wram: [0; WRAM_SIZE],
            io: [0; IO_SIZE],
            hram: [0; HRAM_SIZE],
            ie: 0,
        }
    }
}

// --- UNIT TESTS BEGIN ---

#[cfg(test)]
mod tests {
    use super::*;

    mod gb_mmu_tests {
        use super::*;

        #[test]
        fn test_read() {
            // out of bounds
            let mmu = GbMmu::default();
            let cartridge = GbCartridge::default();
            let ppu = GbPpu::default();
            let addr = 0xFEAB;
            let result = mmu.read(addr, &cartridge, &ppu);
            assert!(result.is_err());

            // TODO: Rewrite this test once the GbCartridge and Mbc objects are implemented
            // rom
            // let gb_mmu = GbMmu {
            //     rom: [0xAB; ROM_SIZE],
            //     ..Default::default()
            // };
            // let addr = 0x1307;
            // let result = gb_mmu.read(addr).unwrap();
            // assert_eq!(result, 0xAB);

            // TODO: Rewrite this test once the GbPpu object is implemented
            // vram
            // let gb_mmu = GbMmu {
            //     vram: [0x11; VRAM_SIZE],
            //     ..Default::default()
            // };
            // let addr = 0x952C;
            // let result = gb_mmu.read(addr).unwrap();
            // assert_eq!(result, 0x11);

            // wram
            let mmu = GbMmu {
                wram: [0x79; WRAM_SIZE],
                ..Default::default()
            };
            let cartridge = GbCartridge::default();
            let ppu = GbPpu::default();
            let addr = 0xC4A2;
            let result = mmu.read(addr, &cartridge, &ppu).unwrap();
            assert_eq!(result, 0x79);

            // eram
            let mmu = GbMmu {
                wram: [0x0D; WRAM_SIZE],
                ..Default::default()
            };
            let cartridge = GbCartridge::default();
            let ppu = GbPpu::default();
            let addr = 0xFA8F;
            let result = mmu.read(addr, &cartridge, &ppu).unwrap();
            assert_eq!(result, 0x0D);
            let addr = 0xFA8F - 0x2000;
            let result = mmu.read(addr, &cartridge, &ppu).unwrap();
            assert_eq!(result, 0x0D);

            // ie
            let mmu = GbMmu {
                ie: 0x1C,
                ..Default::default()
            };
            let cartridge = GbCartridge::default();
            let ppu = GbPpu::default();
            let addr = 0xFFFF;
            let result = mmu.read(addr, &cartridge, &ppu).unwrap();
            assert_eq!(result, 0x1C);
        }

        #[test]
        fn test_write() {
            // out of bounds
            let mut mmu = GbMmu::default();
            let mut cartridge = GbCartridge::default();
            let mut ppu = GbPpu::default();
            let addr = 0xFEAB;
            let data = 0x69;
            let result = mmu.write(addr, data, &mut cartridge, &mut ppu);
            assert!(result.is_err());

            // TODO: Rewrite this test once the GbCartridge and Mbc objects are implemented
            // rom
            // let mut gb_mmu = GbMmu::default();
            // let addr = 0x1307;
            // let data = 0xAB;
            // let result = gb_mmu.write(addr, data);
            // assert!(result.is_ok());
            // let result = gb_mmu.read(addr).unwrap();
            // assert_eq!(result, 0xAB);

            // TODO: Rewrite this test once the GbPpu object is implemented
            // vram
            // let mut gb_mmu = GbMmu::default();
            // let addr = 0x952C;
            // let data = 0x11;
            // let result = gb_mmu.write(addr, data);
            // assert!(result.is_ok());
            // let result = gb_mmu.read(addr).unwrap();
            // assert_eq!(result, 0x11);

            // wram
            let mut mmu = GbMmu::default();
            let mut cartridge = GbCartridge::default();
            let mut ppu = GbPpu::default();
            let addr = 0xC4A2;
            let data = 0x79;
            let result = mmu.write(addr, data, &mut cartridge, &mut ppu);
            assert!(result.is_ok());
            let result = mmu.read(addr, &cartridge, &ppu).unwrap();
            assert_eq!(result, 0x79);

            // eram
            let mut mmu = GbMmu::default();
            let mut cartridge = GbCartridge::default();
            let mut ppu = GbPpu::default();
            let addr = 0xFA8F;
            let data = 0x0D;
            let result = mmu.write(addr, data, &mut cartridge, &mut ppu);
            assert!(result.is_ok());
            let result = mmu.read(addr, &cartridge, &ppu).unwrap();
            assert_eq!(result, 0x0D);
            let addr = 0xFA8F - 0x2000;
            let result = mmu.read(addr, &cartridge, &ppu).unwrap();
            assert_eq!(result, 0x0D);

            // ie
            let mut mmu = GbMmu::default();
            let mut cartridge = GbCartridge::default();
            let mut ppu = GbPpu::default();
            let addr = 0xFFFF;
            let data = 0x1C;
            let result = mmu.write(addr, data, &mut cartridge, &mut ppu);
            assert!(result.is_ok());
            let result = mmu.read(addr, &cartridge, &ppu).unwrap();
            assert_eq!(result, 0x1C);
        }
    }
}

// --- UNIT TESTS END ---
