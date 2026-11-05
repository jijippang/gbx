use super::{CartridgeType, MemoryReadError, MemoryWriteError};
use crate::common::gameboy::memory::common::{Address, Data, OPEN_BUS};

const ROM_BANK_SIZE: usize = 0x4000;
const RAM_BANK_SIZE: usize = 0x2000;

#[derive(Debug, Default)]
// ROM only, no bank switching supported
pub struct Mbc0;

impl Mbc0 {
    fn read(&self, addr: Address, rom: &[Data], _ram: &[Data]) -> Result<Data, MemoryReadError> {
        match addr {
            0x0000..=0x7FFF => Ok(rom.get(addr as usize).copied().unwrap_or(OPEN_BUS)),
            0xA000..=0xBFFF => Err(MemoryReadError::RamNotSupported(OPEN_BUS)),
            _ => Err(MemoryReadError::OutOfBounds(OPEN_BUS)),
        }
    }

    fn write(
        &mut self,
        addr: Address,
        _data: Data,
        _ram: &mut [Data],
    ) -> Result<(), MemoryWriteError> {
        match addr {
            0x0000..=0x7FFF => Err(MemoryWriteError::ReadOnly),
            0xA000..=0xBFFF => Err(MemoryWriteError::RamNotSupported),
            _ => Err(MemoryWriteError::OutOfBounds),
        }
    }
}

#[derive(Debug, Default)]
enum Mbc1Mode {
    #[default]
    // [0x0000, 0x3FFF] and [0xA000, 0xBFFF] are locked to bank 0 of ROM and SRAM respectively
    Simple,
    // [0x0000, 0x3FFF] and [0xA000, 0xBFFF] can be bank-switched via the [0x4000, 0x5FFF] register
    Advanced,
}

#[derive(Debug)]
pub struct Mbc1 {
    // Control registers

    // 5 bit register (Range: [0x01, 0x1F]) that selects the ROM bank number for the [0x4000, 0x7FFF] memory region
    // NOTE: The lowest value that this ROM bank number can be is 0x01 as ROM bank 0x00 is reserved for memory region [0x0000, 0x3FFF]
    rom_bank: usize,

    // 2 bit register (Range: [0x00, 0x03]) that selects the RAM bank number for the [0xA000, 0xBFFF] memory region
    // This register can also be used to extend the ROM bank number by adding 2 more bits (bits 5-6) to the ROM bank number
    ram_bank: usize,

    // Enables or disables reads/writes to the RAM
    ram_enable: bool,

    // Controls the interpretation/behavior of the ram_bank number, as in whether to use it to extend the ROM bank
    // selection or to use it to actively switch between RAM banks [0x00, 0x03]
    mode: Mbc1Mode,
}

impl Mbc1 {
    fn read(&self, addr: Address, rom: &[Data], ram: &[Data]) -> Result<Data, MemoryReadError> {
        match addr {
            // ROM Bank 0
            0x0000..=0x3FFF => {
                let bank = match self.mode {
                    // Simple mode fixes the rom bank to 0
                    Mbc1Mode::Simple => 0,
                    // Advanced mode maps the rom bank to 0x00, 0x20, 0x40, 0x60 depending on the 2 bits of the ram bank
                    // bank = (0b00000) -> rom bank + (0bxx00000) -> ram bank = 0bxx00000
                    Mbc1Mode::Advanced => self.ram_bank << 5,
                };

                let addr = ((bank * ROM_BANK_SIZE) + (addr as usize)) % rom.len();
                Ok(rom.get(addr).copied().unwrap_or(OPEN_BUS))
            }

            // Switchable ROM Banks
            0x4000..=0x7FFF => {
                // Need to account for the that fact that the Mbc1 treats the rom_bank as a 1 if its set to 0
                let low_bits = if self.rom_bank == 0 { 1 } else { self.rom_bank };
                let high_bits = self.ram_bank;

                // Combine the rom bank (lower 5 bits) with the ram bank (upper 2 bits) to form the bank number
                let bank = (high_bits << 5) | low_bits;

                let addr = ((bank * ROM_BANK_SIZE) + ((addr - 0x4000) as usize)) % rom.len();
                Ok(rom.get(addr).copied().unwrap_or(OPEN_BUS))
            }

            // Switchable RAM Banks
            0xA000..=0xBFFF => {
                if !self.ram_enable || ram.is_empty() {
                    return Err(MemoryReadError::RamNotSupported(OPEN_BUS));
                }

                let bank = match self.mode {
                    Mbc1Mode::Simple => 0,
                    Mbc1Mode::Advanced => self.ram_bank,
                };

                let addr = ((bank * RAM_BANK_SIZE) + ((addr - 0xA000) as usize)) % ram.len();
                Ok(ram.get(addr).copied().unwrap_or(OPEN_BUS))
            }

            _ => Err(MemoryReadError::OutOfBounds(OPEN_BUS)),
        }
    }

    fn write(
        &mut self,
        addr: Address,
        data: Data,
        ram: &mut [Data],
    ) -> Result<(), MemoryWriteError> {
        match addr {
            // RAM Enable Register
            0x0000..=0x1FFF => {
                self.ram_enable = (data & 0x0F) == 0x0A;
                Ok(())
            }

            // ROM Bank Register
            0x2000..=0x3FFF => {
                // ROM bank register is only 5 bits wide
                let mut bank = (data & 0x1F) as usize;

                // Need to account for the that fact that the Mbc1 treats the rom_bank as a 1 if its set to 0
                if bank == 0 {
                    bank = 1;
                }

                self.rom_bank = bank;
                Ok(())
            }

            // RAM Bank Register
            0x4000..=0x5FFF => {
                self.ram_bank = (data & 0x03) as usize;
                Ok(())
            }

            // Mode Register
            0x6000..=0x7FFF => {
                self.mode = if data & 0x01 == 0x01 {
                    Mbc1Mode::Advanced
                } else {
                    Mbc1Mode::Simple
                };
                Ok(())
            }

            // RAM
            0xA000..=0xBFFF => {
                if !self.ram_enable || ram.is_empty() {
                    return Err(MemoryWriteError::RamNotSupported);
                }

                let bank = match self.mode {
                    Mbc1Mode::Simple => 0,
                    Mbc1Mode::Advanced => self.ram_bank,
                };

                let addr = ((bank * RAM_BANK_SIZE) + ((addr - 0xA000) as usize)) % ram.len();
                match ram.get_mut(addr) {
                    Some(elem) => *elem = data,
                    None => return Err(MemoryWriteError::OutOfBounds),
                };
                Ok(())
            }

            _ => Err(MemoryWriteError::OutOfBounds),
        }
    }
}

impl Default for Mbc1 {
    fn default() -> Self {
        Self {
            rom_bank: 1,
            ram_bank: 0,
            ram_enable: false,
            mode: Mbc1Mode::Simple,
        }
    }
}

// Memory Bank Controllers
#[derive(Debug)]
pub enum Mbc {
    Mbc0(Mbc0),
    Mbc1(Mbc1),
}

impl Mbc {
    pub fn read(&self, addr: Address, rom: &[Data], ram: &[Data]) -> Result<Data, MemoryReadError> {
        match self {
            Self::Mbc0(mbc) => mbc.read(addr, rom, ram),
            Self::Mbc1(mbc) => mbc.read(addr, rom, ram),
        }
    }

    pub fn write(
        &mut self,
        addr: Address,
        data: Data,
        ram: &mut [Data],
    ) -> Result<(), MemoryWriteError> {
        match self {
            Self::Mbc0(mbc) => mbc.write(addr, data, ram),
            Self::Mbc1(mbc) => mbc.write(addr, data, ram),
        }
    }
}

impl Default for Mbc {
    fn default() -> Self {
        Self::Mbc0(Mbc0::default())
    }
}

impl TryFrom<CartridgeType> for Mbc {
    type Error = String;

    fn try_from(value: CartridgeType) -> Result<Self, Self::Error> {
        match value {
            CartridgeType::Mbc0 => Ok(Self::Mbc0(Mbc0::default())),
            CartridgeType::Mbc1 => Ok(Self::Mbc1(Mbc1::default())),
            _ => Err(format!(
                "Unsupported CartridgeType: {:?}, unable to convert into Mbc",
                value
            )),
        }
    }
}

// --- UNIT TESTS BEGIN ---

#[cfg(test)]
mod tests {
    use super::*;

    mod mbc0_tests {
        use super::*;

        #[test]
        fn test_read() {
            let mbc0 = Mbc0::default();
            let addr = 0x2000;
            let rom = vec![0x99; 1000];
            let ram = vec![0x69; 1000];
            let result = mbc0.read(addr, &rom, &ram);
            assert_eq!(result.unwrap(), OPEN_BUS);

            let mbc0 = Mbc0::default();
            let addr = 0x0249;
            let rom = vec![0x24; 1000];
            let ram = vec![0x81; 1000];
            let result = mbc0.read(addr, &rom, &ram);
            assert_eq!(result.unwrap(), 0x24);

            let mbc0 = Mbc0::default();
            let addr = 0xA1D2;
            let rom = vec![0xCA; 1000];
            let ram = vec![0xDF; 1000];
            let result = mbc0.read(addr, &rom, &ram);
            assert!(matches!(
                result,
                Err(MemoryReadError::RamNotSupported(OPEN_BUS))
            ));

            let mbc0 = Mbc0::default();
            let addr = 0xFFD5;
            let rom = vec![0xEB; 1000];
            let ram = vec![0x88; 1000];
            let result = mbc0.read(addr, &rom, &ram);
            assert!(matches!(
                result,
                Err(MemoryReadError::OutOfBounds(OPEN_BUS))
            ));
        }

        #[test]
        fn test_write() {
            let mut mbc0 = Mbc0::default();
            let addr = 0x101A;
            let data = 0xDB;
            let mut ram = vec![0xAC; 1000];
            let result = mbc0.write(addr, data, &mut ram);
            assert!(matches!(result, Err(MemoryWriteError::ReadOnly)));

            let mut mbc0 = Mbc0::default();
            let addr = 0xAF24;
            let data = 0x76;
            let mut ram = vec![0xDD; 1000];
            let result = mbc0.write(addr, data, &mut ram);
            assert!(matches!(result, Err(MemoryWriteError::RamNotSupported)));

            let mut mbc0 = Mbc0::default();
            let addr = 0xF339;
            let data = 0x92;
            let mut ram = vec![0xEE; 1000];
            let result = mbc0.write(addr, data, &mut ram);
            assert!(matches!(result, Err(MemoryWriteError::OutOfBounds)));
        }
    }

    // mod mbc1_tests {
    //     use super::*;
    //
    //     #[test]
    //     fn test_read() {
    //         todo!();
    //     }
    //
    //     #[test]
    //     fn test_write() {
    //         todo!();
    //     }
    // }
    //
    // mod mbc_tests {
    //     use super::*;
    //
    //     #[test]
    //     fn test_read() {
    //         todo!();
    //     }
    //
    //     #[test]
    //     fn test_write() {
    //         todo!();
    //     }
    // }
}

// --- UNIT TESTS END ---
