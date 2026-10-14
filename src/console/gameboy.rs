use super::Console;
use gb_apu::GbApu;
use gb_cartridge::GbCartridge;
use gb_cpu::GbCpu;
use gb_mmu::GbMmu;
use gb_ppu::GbPpu;
use std::fs;
use std::io;
use std::path::Path;
use tracing::info;

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
    // [64 KiB, 128 KiB, 256 KiB, 512 KiB, 1 MiB]
    const CONSOLE_CART_SIZES: &[usize] = &[0x10000, 0x20000, 0x40000, 0x80000, 0x100000];

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
        info!(
            "Attempting to load cartridge from: {}",
            file_path.as_ref().display()
        );
        let cartridge_data = fs::read(file_path)?;

        // TODO: Perhaps think about moving this validation into the TryFrom trait for the GbCartridge struct
        // Validate cartridge size
        if !Self::CONSOLE_CART_SIZES.contains(&cartridge_data.len()) {
            Err(io::Error::other(format!(
                "Cartridge size: {}, is not one of the allowed cartridge sizes: {:?}",
                cartridge_data.len(),
                Self::CONSOLE_CART_SIZES
            )))
        } else {
            // TODO: Find a better way to handle the Result other than an unwrap()
            self.cartridge = cartridge_data.as_slice().try_into().unwrap();
            Ok(())
        }
    }
}
