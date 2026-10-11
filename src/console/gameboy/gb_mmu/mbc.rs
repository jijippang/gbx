use super::{Address, Data, MmuReadError, MmuWriteError};

// Memory Bank Controllers
pub trait Mbc {
    fn read(&self, addr: Address) -> Result<Data, MmuReadError>;
    fn write(&mut self, addr: Address, data: Data) -> Result<(), MmuWriteError>;
    fn reset(&mut self);
}

#[derive(Debug)]
struct Mbc1 {
    ram_enable: bool,
    mode: bool,
    rom_bank: usize,
    ram_bank: usize,
}

impl Mbc for Mbc1 {
    fn read(&self, addr: Address) -> Result<Data, MmuReadError> {
        todo!();
    }

    fn write(&mut self, addr: Address, data: Data) -> Result<(), MmuWriteError> {
        todo!();
    }

    fn reset(&mut self) {
        todo!();
    }
}
