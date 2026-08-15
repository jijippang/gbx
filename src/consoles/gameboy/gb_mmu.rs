use std::error::Error;
use std::fmt;
use tracing::warn;

pub type Data = u8;
pub type Address = u16;

const OPEN_BUS: Data = 0xFF;
const ROM_SIZE: usize = 0x7FFF - 0x0000 + 1;
const VRAM_SIZE: usize = 0x9FFF - 0x8000 + 1;
const EXTRAM_SIZE: usize = 0xBFFF - 0xA000 + 1;
const WRAM_SIZE: usize = 0xDFFF - 0xC000 + 1;
const OAM_SIZE: usize = 0xFE9F - 0xFE00 + 1;
const IO_SIZE: usize = 0xFF7F - 0xFF00 + 1;
const HRAM_SIZE: usize = 0xFFFE - 0xFF80 + 1;
const IE_SIZE: usize = 0xFFFF - 0xFFFF + 1;

#[derive(Debug)]
pub enum MmuReadError {
    // Attempting to read from memory locations that are unusable or prohibited
    OutOfBounds(Data),

    // Attempting to read from memory locations that are currently locked
    Locked(Data),

    // Attempting to read from memory locations that are write only
    WriteOnly(Data),
}

impl fmt::Display for MmuReadError {
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

impl From<MmuReadError> for Data {
    fn from(err: MmuReadError) -> Self {
        match err {
            MmuReadError::OutOfBounds(data) => data,
            MmuReadError::Locked(data) => data,
            MmuReadError::WriteOnly(data) => data,
        }
    }
}

impl Error for MmuReadError {}

#[derive(Debug)]
pub enum MmuWriteError {
    // Attempting to write to memory locations that are unusable or prohibited
    OutOfBounds,

    // Attempting to write to memory locations that are currently locked
    Locked,

    // Attempting to write to memory locations that are read only
    ReadOnly,
}

impl fmt::Display for MmuWriteError {
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

impl Error for MmuWriteError {}

#[derive(Debug)]
pub struct GbMmu {
    // [0x0000, 0x7FFF]
    rom: [Data; ROM_SIZE],

    // [0x8000, 0x9FFF]
    vram: [Data; VRAM_SIZE],

    // [0xA000, 0xBFFF]
    extram: [Data; EXTRAM_SIZE],

    // [0xC000, 0xDFFF]
    wram: [Data; WRAM_SIZE],

    // [0xE000, 0xFDFF] -> eram (mirror of [0xC000, 0xDDFF] which is a subset of wram)

    // [0xFE00, 0xFE9F]
    oam: [Data; OAM_SIZE],

    // [0xFF00, 0xFF7F]
    io: [Data; IO_SIZE],

    // [0xFF80, 0xFFFE]
    hram: [Data; HRAM_SIZE],

    // [0xFFFF, 0xFFFF]
    ie: [Data; IE_SIZE],
}

impl Default for GbMmu {
    fn default() -> Self {
        Self {
            rom: [0; ROM_SIZE],
            vram: [0; VRAM_SIZE],
            extram: [0; EXTRAM_SIZE],
            wram: [0; WRAM_SIZE],
            oam: [0; OAM_SIZE],
            io: [0; IO_SIZE],
            hram: [0; HRAM_SIZE],
            ie: [0; IE_SIZE],
        }
    }
}

impl GbMmu {
    pub fn read(&self, addr: Address) -> Result<Data, MmuReadError> {
        match addr {
            0x0000..=0x7FFF => Ok(self.rom[addr as usize]),
            0x8000..=0x9FFF => Ok(self.vram[(addr - 0x8000) as usize]),
            0xA000..=0xBFFF => Ok(self.extram[(addr - 0xA000) as usize]),
            0xC000..=0xDFFF => Ok(self.wram[(addr - 0xC000) as usize]),
            0xE000..=0xFDFF => Ok(self.wram[(addr - 0x2000 - 0xC000) as usize]),
            0xFE00..=0xFE9F => Ok(self.oam[(addr - 0xFE00) as usize]),
            0xFF00..=0xFF7F => Ok(self.io[(addr - 0xFF00) as usize]),
            0xFF80..=0xFFFE => Ok(self.hram[(addr - 0xFF80) as usize]),
            0xFFFF..=0xFFFF => Ok(self.ie[(addr - 0xFFFF) as usize]),
            _ => {
                warn!("Invalid Address: {:#X}, cannot read from memory", addr);
                Err(MmuReadError::OutOfBounds(OPEN_BUS))
            }
        }
    }

    pub fn write(&mut self, addr: Address, data: Data) -> Result<(), MmuWriteError> {
        match addr {
            0x0000..=0x7FFF => {
                self.rom[addr as usize] = data;
                Ok(())
            }
            0x8000..=0x9FFF => {
                self.vram[(addr - 0x8000) as usize] = data;
                Ok(())
            }
            0xA000..=0xBFFF => {
                self.extram[(addr - 0xA000) as usize] = data;
                Ok(())
            }
            0xC000..=0xDFFF => {
                self.wram[(addr - 0xC000) as usize] = data;
                Ok(())
            }
            0xE000..=0xFDFF => {
                self.wram[(addr - 0x2000 - 0xC000) as usize] = data;
                Ok(())
            }
            0xFE00..=0xFE9F => {
                self.oam[(addr - 0xFE00) as usize] = data;
                Ok(())
            }
            0xFF00..=0xFF7F => {
                self.io[(addr - 0xFF00) as usize] = data;
                Ok(())
            }
            0xFF80..=0xFFFE => {
                self.hram[(addr - 0xFF80) as usize] = data;
                Ok(())
            }
            0xFFFF..=0xFFFF => {
                self.ie[(addr - 0xFFFF) as usize] = data;
                Ok(())
            }
            _ => {
                warn!(
                    "Invalid Address: {:#X}, cannot write {:#X} to memory",
                    addr, data
                );
                Err(MmuWriteError::OutOfBounds)
            }
        }
    }
}

// Memory Bank Controllers
trait Mbc {}

// --- UNIT TESTS BEGIN ---

#[cfg(test)]
mod tests {
    use super::*;

    mod gb_mmu_tests {
        use super::*;

        #[test]
        fn test_read() {
            // out of bounds
            let gb_mmu = GbMmu::default();
            let addr = 0xFEAB;
            let result = gb_mmu.read(addr);
            assert!(result.is_err());

            // rom
            let gb_mmu = GbMmu {
                rom: [0xAB; ROM_SIZE],
                ..Default::default()
            };
            let addr = 0x1307;
            let result = gb_mmu.read(addr).unwrap();
            assert_eq!(result, 0xAB);

            // vram
            let gb_mmu = GbMmu {
                vram: [0x11; VRAM_SIZE],
                ..Default::default()
            };
            let addr = 0x952C;
            let result = gb_mmu.read(addr).unwrap();
            assert_eq!(result, 0x11);

            // wram
            let gb_mmu = GbMmu {
                wram: [0x79; WRAM_SIZE],
                ..Default::default()
            };
            let addr = 0xC4A2;
            let result = gb_mmu.read(addr).unwrap();
            assert_eq!(result, 0x79);

            // eram
            let gb_mmu = GbMmu {
                wram: [0x0D; WRAM_SIZE],
                ..Default::default()
            };
            let addr = 0xFA8F;
            let result = gb_mmu.read(addr).unwrap();
            assert_eq!(result, 0x0D);
            let addr = 0xFA8F - 0x2000;
            let result = gb_mmu.read(addr).unwrap();
            assert_eq!(result, 0x0D);

            // ie
            let gb_mmu = GbMmu {
                ie: [0x1C; IE_SIZE],
                ..Default::default()
            };
            let addr = 0xFFFF;
            let result = gb_mmu.read(addr).unwrap();
            assert_eq!(result, 0x1C);
        }

        #[test]
        fn test_write() {
            // out of bounds
            let mut gb_mmu = GbMmu::default();
            let addr = 0xFEAB;
            let data = 0x69;
            let result = gb_mmu.write(addr, data);
            assert!(result.is_err());

            // rom
            let mut gb_mmu = GbMmu::default();
            let addr = 0x1307;
            let data = 0xAB;
            let result = gb_mmu.write(addr, data);
            assert!(result.is_ok());
            let result = gb_mmu.read(addr).unwrap();
            assert_eq!(result, 0xAB);

            // vram
            let mut gb_mmu = GbMmu::default();
            let addr = 0x952C;
            let data = 0x11;
            let result = gb_mmu.write(addr, data);
            assert!(result.is_ok());
            let result = gb_mmu.read(addr).unwrap();
            assert_eq!(result, 0x11);

            // wram
            let mut gb_mmu = GbMmu::default();
            let addr = 0xC4A2;
            let data = 0x79;
            let result = gb_mmu.write(addr, data);
            assert!(result.is_ok());
            let result = gb_mmu.read(addr).unwrap();
            assert_eq!(result, 0x79);

            // eram
            let mut gb_mmu = GbMmu::default();
            let addr = 0xFA8F;
            let data = 0x0D;
            let result = gb_mmu.write(addr, data);
            assert!(result.is_ok());
            let result = gb_mmu.read(addr).unwrap();
            assert_eq!(result, 0x0D);
            let addr = 0xFA8F - 0x2000;
            let result = gb_mmu.read(addr).unwrap();
            assert_eq!(result, 0x0D);

            // ie
            let mut gb_mmu = GbMmu::default();
            let addr = 0xFFFF;
            let data = 0x1C;
            let result = gb_mmu.write(addr, data);
            assert!(result.is_ok());
            let result = gb_mmu.read(addr).unwrap();
            assert_eq!(result, 0x1C);
        }
    }
}

// --- UNIT TESTS END ---
