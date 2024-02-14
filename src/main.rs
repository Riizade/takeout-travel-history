use clap::{Parser, Subcommand};
use std::{ffi::OsStr, fs, path::PathBuf};
use zip::ZipArchive;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    List {
        #[arg(short, long)]
        path: PathBuf,
    },
}

fn main() {
    run_cli();
}

fn run_cli() {
    let cli = Cli::parse();
    match &cli.command {
        Some(Commands::List { path }) => {
            if path.extension() == Some(OsStr::new("zip")) {
                let file = fs::File::open(path).unwrap();
                let bufreader = std::io::BufReader::new(file);
                let mut archive = ZipArchive::new(bufreader).unwrap();
                let records_file = archive.by_name("Takeout/Location History (Timeline)/Records.json").unwrap_or_else(|e| panic!("could not extract data from Records.json: {e}"));
                
            } else if path.extension() == Some(OsStr::new("json")) {
            } else {
                let ext = path.extension().unwrap().to_str().unwrap();
                println!(
                    "could not handle unknown filetype, must be one of {{.zip, .json}}: {ext}"
                );
            }
        }
        None => {}
    }
}
