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
        #[arg(long)]
        make: Option<String>,

        #[arg(long)]
        delete: Option<String>,

        #[arg(long)]
        open: Option<String>,

        #[arg(long)]
        new_ver: Option<String>,

        #[arg(long)]
        del_ver: Option<String>,

        #[arg(long)]
        force: bool,

        #[arg(long)]
        record: Option<String>
    },
}

pub fn init(){
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => match qb::init::init() {
            Ok(_) => println!("Init successful"),
            Err(e) => eprintln!("Init failed: {}", e),
        },
        Commands::Qb {make, delete, open, new_ver, del_ver, force, record} => {
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
                match qb::qbox::Qbox::new(&open) {
                    Ok(mut new_qb) => {
                        match new_qb.open() {
                            Ok(q) => {
                                if let Some(new_ver) = new_ver {
                                    match q.new_version(&new_ver) {
                                        Ok(_) => println!("version {} created", new_ver),
                                        Err(e) => println!("Failed to create new version: {}", e),
                                    };                           
                                }
                                if let Some(del_ver) = del_ver {                            
                                    match q.remove_version(&del_ver, force) {
                                        Ok(_) => println!("version {} deleted", del_ver),
                                        Err(e) => println!("Failed to delete version: {}", e),
                                    }
                                }
                                if let Some(record) = record {
                                    q.record(&record)
                                    // match q.record(&record) {
                                        // Ok(_) => println!("d"),
                                        // Err(e) => println!("{}", e),
                                    // };
                                }
                            },
                            Err(e) => {
                                println!("Failed to open qbox: {}", e)
                            },
                        }
                    },
                    Err(e) => {
                        eprintln!("Failed to init qbox: {}", e)
                    }
                }
            }
        }
    }
}