
use clap::ValueEnum;

pub mod gameboy;



#[derive(Debug, Default, Copy, Clone, ValueEnum)]
pub enum ConsoleModel
{
    #[default]
    GameBoy,
    GameBoyColor,
    GameBoyAdvance,
}

pub trait Console
{
    fn step(&mut self, cycles: usize);
    fn read(&self, address: u32) -> u8;
    fn write(&mut self, adddress: u32, value: u8);
    fn get_video_buffer(&self) -> &[u8];
}

