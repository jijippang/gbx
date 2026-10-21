use super::gb_mmu::{MemoryReadError, MemoryWriteError};
use crate::common::gameboy::memory::types::{Address, Data};

pub const VRAM_SIZE: usize = 0x9FFF - 0x8000 + 1;
pub const OAM_SIZE: usize = 0xFE9F - 0xFE00 + 1;

#[derive(Debug)]
pub struct GbPpu {
    // [0x8000, 0x9FFF]
    vram: [Data; VRAM_SIZE],

    // [0xFE00, 0xFE9F]
    oam: [Data; OAM_SIZE],
}

impl GbPpu {
    // TODO: Fill out methods for GbPpu here

    pub fn read(&self, addr: Address) -> Result<Data, MemoryReadError> {
        todo!();
    }

    pub fn write(&mut self, addr: Address, data: Data) -> Result<(), MemoryWriteError> {
        todo!();
    }

    pub fn reset(&mut self) {
        todo!();
    }
}

impl Default for GbPpu {
    fn default() -> Self {
        Self {
            vram: [0; VRAM_SIZE],
            oam: [0; OAM_SIZE],
        }
    }
}
