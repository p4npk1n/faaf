use std::path::PathBuf;
use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Firmware root directory
    #[arg(short, long)]
    firmware_root_dir: PathBuf,

    /// Analyzer directory
    #[arg(short, long)]
    script_directory: PathBuf,

    /// Config file for the analyzer
    #[arg(short, long)]
    config_file: PathBuf,

    /// Output database file(sqlite)
    #[arg(short, long)]
    database_file: PathBuf,
}

fn main() {
    let args = Args::parse();

    let firmware_root_directory = args.firmware_root_dir.canonicalize().expect("Failed to convert to absolute path");

    if !args.script_directory.is_dir() {
        panic!("Script Directory is not a valid directory");
    }

    if !args.config_file.is_file() {
        panic!("Config File is not a valid file");
    }

    if args.database_file.exists() && !args.database_file.is_file() {
        panic!("Database File path exists and is not a file");
    }

    let result = faaf::gateway::gateway::analyze(&firmware_root_directory, &args.script_directory, &args.config_file, &args.database_file);
    println!("{:?}", result)
}