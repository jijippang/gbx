use crate::consoles::Console;
use tracing::{info, warn};

type SpeedMultiplier = f64;

const MIN_EMU_SPD_MULT: SpeedMultiplier = 0.1;
const MAX_EMU_SPD_MULT: SpeedMultiplier = 100.0;

#[derive(Debug)]
pub enum EmulatorSpeedPreset {
    // 0.1x
    Min,

    // 0.5x
    Half,

    // 1.0x
    Normal,

    // 2.0x
    Double,

    // 10.0x
    Turbo,

    // 100.0x
    Max,
}

impl EmulatorSpeedPreset {
    fn to_speed_mult(&self) -> SpeedMultiplier {
        match self {
            Self::Min => MIN_EMU_SPD_MULT,
            Self::Half => 0.5,
            Self::Normal => 1.0,
            Self::Double => 2.0,
            Self::Turbo => 10.0,
            Self::Max => MAX_EMU_SPD_MULT,
        }
    }
}

#[derive(Debug)]
pub enum EmulatorSpeed {
    Preset(EmulatorSpeedPreset),
    Custom(SpeedMultiplier),
}

impl EmulatorSpeed {
    fn to_speed_mult(&self) -> SpeedMultiplier {
        match self {
            Self::Preset(preset) => preset.to_speed_mult(),
            Self::Custom(custom) => {
                let custom_val = *custom;
                let clamped = custom_val.clamp(MIN_EMU_SPD_MULT, MAX_EMU_SPD_MULT);
                if clamped != custom_val {
                    warn!(
                        "Custom EmulatorSpeed value: {} is outside of valid range: [{}, {}], clamping value to {}",
                        custom, MIN_EMU_SPD_MULT, MAX_EMU_SPD_MULT, clamped
                    );
                }
                clamped
            }
        }
    }
}

#[derive(Debug)]
pub struct Emulator<C: Console> {
    console: C,
    emu_speed: EmulatorSpeed,
}

impl<C: Console> Emulator<C> {
    pub fn new(console: C, emu_speed: EmulatorSpeed) -> Self {
        Self {
            console: console,
            emu_speed: emu_speed,
        }
    }

    pub fn run(&mut self) {
        info!("Running Emulator");

        let cycles_per_frame = ((C::get_cycles_per_frame() as SpeedMultiplier)
            * self.emu_speed.to_speed_mult())
        .round() as u64;
        self.console.step(cycles_per_frame);
        // self.generate_frame();
        // self.handle_input();
    }

    fn generate_frame(&mut self, video_buffer: &[u8]) {
        info!(
            "Generating frame with video buffer size: {}",
            video_buffer.len()
        );
        todo!();
    }
}
