use super::Console;
use crate::common::gameboy::cpu::common::TCycle;
use gb_apu::GbApu;
use gb_cartridge::GbCartridge;
use gb_cpu::GbCpu;
use gb_mmu::GbMmu;
use gb_ppu::GbPpu;
use tracing::{info, warn};

mod gb_apu;
mod gb_cartridge;
mod gb_cpu;
mod gb_mmu;
mod gb_ppu;

#[derive(Debug, Default)]
pub struct GameBoy {
    apu: GbApu,
    cpu: GbCpu,
    ppu: GbPpu,
    mmu: GbMmu,
    cartridge: GbCartridge,

    // Keeps track of how many TCycles the CPU has stepped beyond the frame boundary
    // This is necessary in order to keep an accurate time-sync between the rest of the components and the CPU
    catch_up_cycles: TCycle,
}

impl Console for GameBoy {
    const CONSOLE_MASTER_CLK_FREQ: f64 = 4.194304e6;
    const CONSOLE_FPS: f64 = 59.727500569606;

    type Cartridge = GbCartridge;

    fn step(&mut self, cycles: u64) {
        // Add any leftover cycles from the last frame to our current frame's cycles
        let mut remaining_cycles = cycles + self.catch_up_cycles;
        // Reset the catch_up_cycles for the next time the CPU steps beyond the frame boundary
        self.catch_up_cycles = 0;

        while remaining_cycles > 0 {
            let t_cycles = self
                .cpu
                .step(&mut self.mmu, &mut self.cartridge, &mut self.ppu);

            // Check if the number of t-cycles the CPU took was more than the cycles we have remaining in this frame
            match remaining_cycles.checked_sub(t_cycles) {
                Some(new_remaining_cycles) => {
                    // Underflow did not happen, step the PPU and the APU by the same amount of t-cycles to synchronize them with the CPU
                    // self.ppu.step(t_cycles);
                    // self.apu.step(t_cycles);
                    remaining_cycles = new_remaining_cycles
                }
                None => {
                    // Underflow happened, store the remainder into the catch_up_cycles for next time
                    self.catch_up_cycles = t_cycles - remaining_cycles;
                    break;
                }
            }
        }
    }

    fn get_video_buffer(&self) -> &[u8] {
        todo!();
        // self.ppu.get_video_buffer();
    }

    fn load_cartridge(&mut self, cartridge: Self::Cartridge) {
        self.cartridge = cartridge
    }
}
