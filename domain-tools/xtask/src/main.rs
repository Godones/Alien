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
    Build {
        /// The name of the domain project
        #[arg(short, long, value_name = "NAME")]
        name: String,
        /// The log level, default is INFO
        #[arg(short, long, value_name = "LOG", default_value = "INFO")]
        log: String,
    },
    BuildAll {
        /// The log level, default is INFO
        #[arg(short, long, value_name = "LOG", default_value = "INFO")]
        log: String,
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
        Some(Commands::BuildAll { log }) => {
            println!("Building all domain projects, LOG: {log}");
            subcommand::build::build_all(log);
        }
        Some(Commands::Build { name, log }) => {
            println!("Building domain project: {}, LOG: {}", name, log);
            subcommand::build::build_domain(name, log);
        }
        None => {}
    }
}
