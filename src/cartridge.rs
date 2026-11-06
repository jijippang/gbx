use std::error::Error;
use std::ops::RangeInclusive;
use std::path::Path;

pub trait Cartridge: Sized {
    const CARTRIDGE_SIZE_RANGE: RangeInclusive<usize>;
    const CARTRIDGE_PAD_VAL: Self::Data;

    type Address;
    type Data;
    type ReadError: Error;
    type WriteError: Error;

    fn new(source_path: impl AsRef<Path>) -> Result<Self, String>;
    fn read(&self, addr: Self::Address) -> Result<Self::Data, Self::ReadError>;
    fn write(&mut self, addr: Self::Address, data: Self::Data) -> Result<(), Self::WriteError>;
    fn source_path(&self) -> &Path;
}
