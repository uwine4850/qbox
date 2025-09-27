use clap::{Parser, Subcommand};
use crate::qb::{self, error::QboxError};

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
        #[command(subcommand)]
        cmd: QbCommands,
    }
}

#[derive(Subcommand)]
enum QbCommands {
    Make {name: String},    
    Delete {name: String},
    Open {
        name: String,
        #[command(subcommand)]
        actions: QbActions
    }
}

#[derive(Subcommand)]
enum QbActions {
    NewVer { name: String },
    DelVer {
        name: String,
    
        #[arg(long)]
        force: bool
    },
    Record {
        name: String,
        
        #[arg(long)]
        force: bool
    },
    Backup,
    Apply { name: String },
}

fn command_result<T, E: std::fmt::Display>(res: Result<T, E>, success: &str, failure: &str) {
    match res {
        Ok(_) => println!("{}", success),
        Err(e) => eprintln!("{}: {}", failure, e),
    }
}

fn open_qbox(name: &str) -> Result<qb::qbox::Qbox, QboxError>{
    match qb::qbox::Qbox::new(name) {
        Ok(mut qbox) => {
            match qbox.open() {
                Ok(_) => {
                    Ok(qbox)
                },
                Err(e) => {
                    Err(e)
                },
            }
        },
        Err(e) => {
            Err(e)
        }
    }
}

pub fn init() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => {
            command_result(qb::init::init(), "qb init success", "error qb init");
        }
        Commands::Qb { cmd } => {
            match cmd {
                QbCommands::Make { name } => {
                    command_result(qb::qbox::make(name.as_str()), &format!("Created {}", name), "Failed to create");
                }
                QbCommands::Delete { name } => {
                    command_result(qb::qbox::delete(name.as_str()), &format!("Deleted {}", name), "Failed to delete");
                }
                QbCommands::Open { name, actions } => {
                    let open_qbox: qb::qbox::Qbox = match open_qbox(name.as_str()) {
                        Ok(qbox) => {qbox},
                        Err(e) => {
                            eprintln!("Failed to open qbox: {}", e);
                            return;
                        }
                    };
                    
                    match actions {
                        QbActions::NewVer { name: ver } => {
                            command_result(open_qbox.new_version(ver.as_str()), &format!("New version {} created in {}", ver, name), "Failed to create version");
                        }
                        QbActions::DelVer { name: ver, force } => {
                            command_result(open_qbox.remove_version(ver.as_str(), force), &format!("Deleted version {} from {} (force={})", ver, name, force), "Failed to delete version");
                        }
                        QbActions::Record { name: ver, force } => {
                            command_result(open_qbox.record(ver.as_str(), force), &format!("Recorded version {} in {} (force={})", ver, name, force), "Failed to record version");
                        }
                        QbActions::Backup => {
                            command_result(open_qbox.make_backup(), &format!("Backup created for {}", name), "Failed to create backup");
                        }
                        QbActions::Apply { name: ver } => {
                            command_result(open_qbox.apply(ver.as_str()), &format!("Applied version {} to {}", ver, name), "Failed to apply version");
                        }
                    }
                }
            }
        }
    }
}
