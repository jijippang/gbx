
use std::path::PathBuf;
use clap::Parser;
use emulator::Emulator;
use consoles::ConsoleModel;
use consoles::{gameboy::GameBoy};

mod emulator;
mod consoles;




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
}




fn main() 
{
    let args = Args::parse();
    // println!("rom_path: {:?}", args.rom_path);
    // println!("console_model: {:?}", args.console_model);




    let console = match args.console_model
    {
        ConsoleModel::GameBoy => GameBoy::default(),
        // ConsoleModel::GameBoyColor => GameBoyColor::default(),
        // ConsoleModel::GameBoyAdvance => GameBoyAdvance::default(),
        _ => panic!("Console Model: {:?}, is not yet supported", args.console_model),
    };

    let emulator = Emulator::new(console);
    println!("{:?}", emulator);
}



