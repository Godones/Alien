mod subcommand;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Generates a new domain project
    New {
        /// The name of the domain project
        #[arg(short, long, value_name = "NAME")]
        name: String,
    },
}

fn main() {
    let cli = Cli::parse();

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command {
        Some(Commands::New { name }) => {
            println!("Creating new domain project: {}", name);
            subcommand::new::create_domain(name);
        }
        None => {}
    }
}
