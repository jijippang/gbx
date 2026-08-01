
use std::path::Path;
use crate::consoles::Console;
use gb_cpu::GbCpu;
use gb_mmu::GbMmu;
use gb_ppu::GbPpu;
use gb_apu::GbApu;

mod gb_cpu;
mod gb_mmu;
mod gb_ppu;
mod gb_apu;


#[derive(Debug, Default)]
pub struct GameBoy
{
    cpu: GbCpu,
    mmu: GbMmu,
    ppu: GbPpu,
    apu: GbApu,
}


impl Console for GameBoy
{
    fn step(&mut self, cycles: usize)
    {

    }

    fn read_memory(&self, addr: u32) -> u8
    {
        0
    }

    fn write_memory(&mut self, addr: u32, val: u8)
    {

    }

    fn get_video_buffer(&self) -> &[u8]
    {
        &[0]
    }

    fn read_rom(&mut self, file_path: &Path)
    {

    }
}


