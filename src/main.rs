use cartridge::Cartridge;
use clap::{Parser, ValueEnum};
use console::gameboy::GameBoy;
use console::{Console, ConsoleModel};
use emulator::{Emulator, EmulatorSpeed, EmulatorSpeedPreset};
use std::fs::File;
use std::path::PathBuf;
use tracing::{info, level_filters::LevelFilter};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

mod cartridge;
mod common;
mod console;
mod emulator;

#[derive(Debug, Default, Copy, Clone, ValueEnum)]
enum LogLevelFilter {
    Trace,
    Debug,
    #[default]
    Info,
    Warn,
    Error,
    Off,
}

impl From<LogLevelFilter> for LevelFilter {
    fn from(value: LogLevelFilter) -> Self {
        match value {
            LogLevelFilter::Trace => Self::TRACE,
            LogLevelFilter::Debug => Self::DEBUG,
            LogLevelFilter::Info => Self::INFO,
            LogLevelFilter::Warn => Self::WARN,
            LogLevelFilter::Error => Self::ERROR,
            LogLevelFilter::Off => Self::OFF,
        }
    }
}

impl From<LevelFilter> for LogLevelFilter {
    fn from(value: LevelFilter) -> Self {
        match value {
            LevelFilter::TRACE => Self::Trace,
            LevelFilter::DEBUG => Self::Debug,
            LevelFilter::INFO => Self::Info,
            LevelFilter::WARN => Self::Warn,
            LevelFilter::ERROR => Self::Error,
            LevelFilter::OFF => Self::Off,
        }
    }
}

fn init_tracing(
    log_level_filter: LogLevelFilter,
    log_dir_path: Option<PathBuf>,
) -> Option<WorkerGuard> {
    let log_level_filter = LevelFilter::from(log_level_filter);

    let filter = EnvFilter::default().add_directive(log_level_filter.into());

    let console_layer = fmt::layer().with_writer(std::io::stdout);

    const LOG_FILE_PREFIX: &str = "gbx";
    const LOG_FILE_SUFFIX: &str = "log";
    let (file_layer, guard) = if let Some(path) = log_dir_path {
        let path = path.join(format!("{}.{}", LOG_FILE_PREFIX, LOG_FILE_SUFFIX));
        let file = File::create(path).expect("Failed to create log file");

        let (non_blocking, guard) = tracing_appender::non_blocking(file);
        let file_layer = fmt::layer().with_writer(non_blocking).with_ansi(false);

        (Some(file_layer), Some(guard))
    } else {
        (None, None)
    };

    tracing_subscriber::registry()
        .with(filter)
        .with(console_layer)
        .with(file_layer)
        .init();

    info!(
        "Tracing initialized with log level filter: {}",
        log_level_filter
    );

    guard
}

#[derive(Parser, Debug)]
#[command(version, name = "gbx", about = "Emulator for the Game Boy family")]
struct Args {
    /// Path to the ROM file to load
    #[arg(short, long)]
    rom_path: PathBuf,

    /// Which console model to emulate
    #[arg(short, long, value_enum, default_value_t = ConsoleModel::GameBoy)]
    console_model: ConsoleModel,

    /// Log level filter
    #[arg(short, long, value_enum, default_value_t = LogLevelFilter::Info)]
    log_level_filter: LogLevelFilter,

    /// Path to the log directory where logs are written to
    #[arg(short = 'p', long)]
    log_dir_path: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();

    // Initialize logging through tracing
    let _guard = init_tracing(args.log_level_filter, args.log_dir_path);

    info!("Starting GBX Emulator");

    let cartridge = Cartridge::new(args.rom_path).unwrap();
    // TODO: Find a better way to handle the console creation
    let mut console = match args.console_model {
        ConsoleModel::GameBoy => GameBoy::default(),
        // ConsoleModel::GameBoyColor => GameBoyColor::default(),
        // ConsoleModel::GameBoyAdvance => GameBoyAdvance::default(),
        _ => panic!(
            "Console Model: {:?} is not yet supported",
            args.console_model
        ),
    };
    console.load_cartridge(cartridge);

    let mut emulator = Emulator::new(console, EmulatorSpeed::Preset(EmulatorSpeedPreset::Normal));
    // let mut emulator = Emulator::new(console, EmulatorSpeed::Custom(4.0));
    emulator.run();
}
