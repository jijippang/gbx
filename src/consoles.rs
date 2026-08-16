use clap::ValueEnum;
use std::io;
use std::path::Path;

pub mod gameboy;

type Cartridge = Vec<u8>;

#[derive(Debug, Default, Copy, Clone, ValueEnum)]
pub enum ConsoleModel {
    #[default]
    GameBoy,
    GameBoyColor,
    GameBoyAdvance,
}

pub trait Console {
    const CONSOLE_MASTER_CLK_FREQ: f64;
    const CONSOLE_FPS: f64;
    const CONSOLE_CART_SIZES: &[usize];

    fn step(&mut self, cycles: u64);
    fn get_video_buffer(&self) -> &[u8];
    fn load_cartridge(&mut self, file_path: &Path) -> io::Result<Cartridge>;
    fn get_cycles_per_frame() -> u64;
}
