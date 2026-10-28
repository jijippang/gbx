use super::Console;
use crate::common::gameboy::memory::types::Data;
use gb_apu::GbApu;
use gb_cartridge::GbCartridge;
use gb_cpu::GbCpu;
use gb_mmu::GbMmu;
use gb_ppu::GbPpu;
use std::fs;
use std::io;
use std::ops::RangeInclusive;
use std::path::Path;
use tracing::{info, warn};

mod gb_apu;
mod gb_cartridge;
mod gb_cpu;
mod gb_mmu;
mod gb_ppu;

// Timing Cycles (There are 4 timing cycles in 1 machine cycle), and each timing cycle should last approx. 1 / 4.194 [MHz] ~= 238.4 [ns]
type TCycles = u64;

#[derive(Debug, Default)]
pub struct GameBoy {
    apu: GbApu,
    cpu: GbCpu,
    ppu: GbPpu,
    mmu: GbMmu,
    cartridge: GbCartridge,

    // Keeps track of how many TCycles the CPU has stepped beyond the frame boundary
    // This is necessary in order to keep an accurate time-sync between the rest of the components and the CPU
    catch_up_cycles: TCycles,
}

impl Console for GameBoy {
    const CONSOLE_MASTER_CLK_FREQ: f64 = 4.194304e6;
    const CONSOLE_FPS: f64 = 59.727500569606;
    // [32 KiB, 8 MiB]
    const CONSOLE_CART_SIZE_RANGE: RangeInclusive<usize> = 0x8000..=0x800000;

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

    fn load_cartridge(&mut self, file_path: impl AsRef<Path>) -> Result<(), io::Error> {
        const CART_PAD_VAL: Data = 0xFF;

        info!(
            "Attempting to load cartridge from: {}",
            file_path.as_ref().display()
        );
        let mut cartridge_data = fs::read(file_path)?;

        // TODO: Perhaps think about moving this validation into the TryFrom trait for the GbCartridge struct
        // Validate cartridge size
        let cartridge_data_len = cartridge_data.len();
        if !Self::CONSOLE_CART_SIZE_RANGE.contains(&cartridge_data_len) {
            Err(io::Error::other(format!(
                "Cartridge size: {}, does not fit within the allowed cartridge size range: {:?}",
                cartridge_data_len,
                Self::CONSOLE_CART_SIZE_RANGE
            )))
        } else {
            // If the cartridge is not a power of 2, pad to the nearest power of 2 with CART_PAD_VAL
            let target_size = cartridge_data_len.next_power_of_two();
            if cartridge_data_len != target_size {
                warn!(
                    "Cartridge size: {} is not a power of 2, padding to {} bytes with value: {:#X}",
                    cartridge_data_len, target_size, CART_PAD_VAL
                );
                cartridge_data.resize(target_size, CART_PAD_VAL);
            }

            // TODO: Find a better way to handle the Result other than an unwrap()
            self.cartridge = cartridge_data.as_slice().try_into().unwrap();
            Ok(())
        }
    }
}
