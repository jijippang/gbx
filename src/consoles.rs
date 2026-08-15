use clap::ValueEnum;
use std::path::Path;

pub mod gameboy;

#[derive(Debug, Default, Copy, Clone, ValueEnum)]
pub enum ConsoleModel {
    #[default]
    GameBoy,
    GameBoyColor,
    GameBoyAdvance,
}

pub trait Console {
    fn step(&mut self, cycles: usize);
    fn read_memory(&self, addr: u32) -> u8;
    fn write_memory(&mut self, addr: u32, val: u8);
    fn get_video_buffer(&self) -> &[u8];
    fn read_rom(&mut self, file_path: &Path);
}
