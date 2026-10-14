use super::{Address, Data, MemoryReadError, MemoryWriteError};

// Memory Bank Controllers
#[derive(Debug)]
pub enum Mbc {
    // ROM and RAM only, no bank switching supported
    Mbc0 {
        rom: Vec<Data>,
        ram: Vec<Data>,
        ram_enable: bool,
    },

    Mbc1 {
        rom: Vec<Data>,
        ram: Vec<Data>,
        rom_bank: usize,
        ram_bank: usize,
        ram_enable: bool,
    },
}

impl Mbc {
    fn read(&self, addr: Address) -> Result<Data, MemoryReadError> {
        todo!();
    }

    fn write(&mut self, addr: Address, data: Data) -> Result<(), MemoryWriteError> {
        todo!();
    }

    fn reset(&mut self) {
        todo!();
    }
}

impl Default for Mbc {
    fn default() -> Self {
        Self::Mbc0 {
            rom: Default::default(),
            ram: Default::default(),
            ram_enable: Default::default(),
        }
    }
}
