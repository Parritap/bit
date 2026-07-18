mod internals;
mod phases;
use clap::{Parser, Subcommand};
use internals::objects;
use std::{process::ExitCode, string};

#[derive(Parser)]
#[command(name = "git", version, about = "A simplified version control system")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    // Inits a bit repository
    Init,

    /// Clone a repository into a new directory
    Clone {
        /// The repository URL to clone (Positional)
        repository: String,
    },

    /// Record changes to the repository
    Commit {
        /// Use the given message as the commit message (Option)
        #[arg(short = 'm', long)]
        message: String,

        /// Stage all modified and deleted files (Flag)
        #[arg(short = 'a', long)]
        all: bool,
    },

    /// Update remote refs along with associated objects
    Push {
        /// The remote repository to push to
        #[arg(default_value = "origin")]
        remote: String,

        /// The branch to push
        #[arg(default_value = "main")]
        branch: String,
    },

    Object {
        #[arg(short = 'c', long)]
        create: String,
    },
}

#[derive(Subcommand)]
enum ObjectCommands {
    Create { path: String },
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Clone { repository } => {
            println!("Cloning repository from: {}", repository);
            ExitCode::SUCCESS
        }
        Commands::Commit { message, all } => {
            if *all {
                println!("Staging all changes...");
            }
            println!("Creating commit with message: '{}'", message);
            ExitCode::SUCCESS
        }
        Commands::Push { remote, branch } => {
            println!("Pushing branch '{}' to remote '{}'", branch, remote);
            ExitCode::SUCCESS
        }
        Commands::Init => phases::init::check_dir(),
        Commands::Object { create } => {
            internals::objects::create(create);
            ExitCode::SUCCESS
        }
    }
}
