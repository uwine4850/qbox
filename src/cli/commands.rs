use clap::{Parser, Subcommand};
use crate::qb;

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
    Qb {
        #[arg(short, long)]
        make: Option<String>,

        #[arg(short, long)]
        delete: Option<String>,

        #[arg(short, long)]
        open: Option<String>,

        #[arg(short, long)]
        new_ver: Option<String>,
    },
}

pub fn init(){
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => match qb::init::init() {
            Ok(_) => println!("Init successful"),
            Err(e) => eprintln!("Init failed: {}", e),
        },
        Commands::Qb {make, delete, open, new_ver} => {
            if let Some(make) = make {
                match qb::qbox::make(&make) {
                    Ok(_) => println!("Qbox \"{}\" created successfully", make),
                    Err(e) => eprintln!("Failed to create qbox: {}", e),
                }
            }
            if let Some(delete) = delete {
                match qb::qbox::delete(&delete) {
                    Ok(_) => println!("Qbox \"{}\" deleted successfully", delete),
                    Err(e) => eprintln!("Failed to delete qbox: {}", e),
                }
            }
            if let Some(open) = open {
                let mut new_qb = qb::qbox::Qbox::new(&open);
                match new_qb.open() {
                    Ok(q) => {
                        if let Some(new_ver) = new_ver {
                            match q.new_version(&new_ver) {
                                Ok(_) => println!("version {} created", new_ver),
                                Err(e) => println!("Failed to create new version: {}", e),
                            };                           
                        }
                    },
                    Err(e) => {
                        println!("Failed to open qbox: {}", e)
                    },
                }
            }
        }
    }
}