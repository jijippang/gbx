use crate::cartridge::Cartridge;
use clap::ValueEnum;

pub mod gameboy;

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

    type Cartridge: Cartridge;

    fn step(&mut self, cycles: u64);
    fn get_video_buffer(&self) -> &[u8];
    fn load_cartridge(&mut self, cartridge: Self::Cartridge);
    fn get_cycles_per_frame() -> u64 {
        (Self::CONSOLE_MASTER_CLK_FREQ / Self::CONSOLE_FPS).round() as u64
    }
}
