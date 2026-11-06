use std::ops::RangeInclusive;
use std::path::Path;

pub trait Cartridge: Sized {
    const CARTRIDGE_SIZE_RANGE: RangeInclusive<usize>;
    // NOTE: Might be better to replace u8 with num::Unsigned from the num crate to be more generic
    const CARTRIDGE_PAD_VAL: u8;

    fn new(source_path: impl AsRef<Path>) -> Result<Self, String>;
    fn source_path(&self) -> &Path;
}
