use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "mpm", about = "A Rust-based package manager for Project Frontier", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: MPMCommand,
}

#[derive(Subcommand)]
enum MPMCommand {
	/// installs all the packages in the workspace
    Install,
	/// publishes the current package
    Publish,
}

fn main() {
    let args: Args = Args::parse();

    match args.command {
        MPMCommand::Install => {
            println!("Installing muse packages");
        }
        MPMCommand::Publish => {
            println!("Publishing muse package");
        }
    }
}
