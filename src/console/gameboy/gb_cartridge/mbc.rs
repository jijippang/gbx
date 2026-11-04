use super::{CartridgeType, MemoryReadError, MemoryWriteError};
use crate::common::gameboy::memory::common::{Address, Data, OPEN_BUS};

#[derive(Debug, Default)]
// ROM only, no bank switching supported
pub struct Mbc0;

impl Mbc0 {
    fn read(&self, addr: Address, rom: &[Data], _ram: &[Data]) -> Result<Data, MemoryReadError> {
        match addr {
            0x0000..=0x7FFF => Ok(rom.get(addr as usize).copied().unwrap_or(OPEN_BUS)),
            0xA000..=0xBFFF => Err(MemoryReadError::RamNotSupported(OPEN_BUS)),
            _ => Ok(OPEN_BUS),
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
            _ => Ok(()),
        }
    }
}

#[derive(Debug, Default)]
enum Mbc1Mode {
    #[default]
    Simple,
    Advanced,
}

#[derive(Debug)]
pub struct Mbc1 {
    rom_bank: usize,
    ram_bank: usize,
    ram_enable: bool,
    banking_mode: Mbc1Mode,
}

impl Mbc1 {
    fn read(&self, addr: Address, rom: &[Data], ram: &[Data]) -> Result<Data, MemoryReadError> {
        todo!();
    }

    fn write(
        &mut self,
        addr: Address,
        data: Data,
        ram: &mut [Data],
    ) -> Result<(), MemoryWriteError> {
        todo!();
    }
}

impl Default for Mbc1 {
    fn default() -> Self {
        Self {
            rom_bank: 1,
            ram_bank: 0,
            ram_enable: false,
            banking_mode: Mbc1Mode::default(),
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
