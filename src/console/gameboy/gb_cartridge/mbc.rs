use super::{Address, CartridgeType, Data, MemoryReadError, MemoryWriteError};

#[derive(Debug, Default)]
// ROM and RAM only, no bank switching supported
pub struct Mbc0 {
    rom: Vec<Data>,
    ram: Vec<Data>,
    ram_enable: bool,
}

impl Mbc0 {
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

#[derive(Debug, Default)]
pub struct Mbc1 {
    rom: Vec<Data>,
    ram: Vec<Data>,
    rom_bank: usize,
    ram_bank: usize,
    ram_enable: bool,
    banking_mode: bool,
}

impl Mbc1 {
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

// Memory Bank Controllers
#[derive(Debug)]
pub enum Mbc {
    Mbc0(Mbc0),
    Mbc1(Mbc1),
}

impl Mbc {
    pub fn read(&self, addr: Address) -> Result<Data, MemoryReadError> {
        match self {
            Self::Mbc0(mbc) => mbc.read(addr),
            Self::Mbc1(mbc) => mbc.read(addr),
        }
    }

    pub fn write(&mut self, addr: Address, data: Data) -> Result<(), MemoryWriteError> {
        match self {
            Self::Mbc0(mbc) => mbc.write(addr, data),
            Self::Mbc1(mbc) => mbc.write(addr, data),
        }
    }

    pub fn reset(&mut self) {
        match self {
            Self::Mbc0(mbc) => mbc.reset(),
            Self::Mbc1(mbc) => mbc.reset(),
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
