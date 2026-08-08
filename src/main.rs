
use std::path::PathBuf;
use clap::{Parser, ValueEnum};
use tracing::{level_filters::LevelFilter, info};
use tracing_subscriber::EnvFilter;
use emulator::Emulator;
use consoles::ConsoleModel;
use consoles::{gameboy::GameBoy};

mod emulator;
mod consoles;





#[derive(Debug, Default, Copy, Clone, ValueEnum)]
enum LogLevelFilter
{
    Trace,
    Debug,
    #[default]
    Info,
    Warn,
    Error,
    Off,
}


impl From<LogLevelFilter> for LevelFilter
{
    fn from(level: LogLevelFilter) -> Self
    {
        match level
        {
            LogLevelFilter::Trace => Self::TRACE,
            LogLevelFilter::Debug => Self::DEBUG,
            LogLevelFilter::Info => Self::INFO,
            LogLevelFilter::Warn => Self::WARN,
            LogLevelFilter::Error => Self::ERROR,
            LogLevelFilter::Off => Self::OFF,
        }
    }
}


impl From<LevelFilter> for LogLevelFilter
{
    fn from(level: LevelFilter) -> Self
    {
        match level
        {
            LevelFilter::TRACE => Self::Trace,
            LevelFilter::DEBUG => Self::Debug,
            LevelFilter::INFO => Self::Info,
            LevelFilter::WARN => Self::Warn,
            LevelFilter::ERROR => Self::Error,
            LevelFilter::OFF => Self::Off,
        }
    }
}


#[derive(Parser, Debug)]
#[command(
    version,
    name = "gbx", 
    about = "Emulator for the Game Boy family",
)]
struct Args
{
    /// Path to the ROM file to load
    #[arg(short, long)]
    rom_path: Option<PathBuf>,

    /// Which console model to emulate
    #[arg(short, long, value_enum, default_value_t = ConsoleModel::GameBoy)]
    console_model: ConsoleModel,

    /// Log level filter
    #[arg(short, long, value_enum, default_value_t = LogLevelFilter::Info)]
    log_level_filter: LogLevelFilter,

    /// Path to the log directory where logs are written to
    #[arg(short = 'p', long)]
    log_path: Option<PathBuf>,
}


fn main() 
{
    let args = Args::parse();
    // println!("rom_path: {:?}", args.rom_path);
    // println!("console_model: {:?}", args.console_model);


    // Initialize logging through tracing
    let log_level_filter = LevelFilter::from(args.log_level_filter);
    let filter = EnvFilter::default().add_directive(log_level_filter.into());

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .init();






    info!("Starting GBX Emulator");
    info!("Log Level Filter: {}", log_level_filter);



    let console = match args.console_model
    {
        ConsoleModel::GameBoy => GameBoy::default(),
        // ConsoleModel::GameBoyColor => GameBoyColor::default(),
        // ConsoleModel::GameBoyAdvance => GameBoyAdvance::default(),
        _ => panic!("Console Model: {:?}, is not yet supported", args.console_model),
    };

    let emulator = Emulator::new(console);
    // println!("{:?}", emulator);
}


