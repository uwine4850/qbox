use clap::{Parser, Subcommand};
use crate::qbox;

#[derive(Parser)]
#[command(name = "myapp")]
#[command(about = "qbox cli", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init,
}

pub fn init(){
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => match qbox::init::init() {
                Ok(_) => println!("Init successful"),
                Err(e) => eprintln!("Init failed: {}", e),
            },
    }
}