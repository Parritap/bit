mod constants;
mod internals;
mod phases;
mod utils;
use ObjectCommands as obj_cmds;
use clap::{Parser, Subcommand};
use internals::objects;
use std::io;
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "git", version, about = "A simplified version control system")]
struct Cli {
    #[command(subcommand)]
    command: BitCommands,
}

#[derive(Subcommand)]
enum BitCommands {
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
    HashObject {
        path: String,
        #[arg(short = 'w', long)]
        write: bool,
    },
    CatFile {
        hash: String,
        #[arg(short = 'p')]
        print: bool,
    },
}

#[derive(Subcommand)]
enum ObjectCommands {
    /// Create an object from a file
    #[command(name = "hash-object")]
    Hash {},
    /// Read an object by its hash
    #[command(name = "cat-object")]
    CatObject { hash: String },
}

fn main() -> ExitCode {
    match _main() {
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => {
            println!("{:?}", err);
            ExitCode::FAILURE
        }
    }
}

fn _main() -> Result<(), io::Error> {
    let cli = Cli::parse();

    match &cli.command {
        BitCommands::Clone { repository } => {
            println!("Cloning repository from: {}", repository);
            Ok(())
        }
        BitCommands::Commit { message, all } => {
            if *all {
                println!("Staging all changes...");
            }
            println!("Creating commit with message: '{}'", message);
            Ok(())
        }
        BitCommands::Push { remote, branch } => {
            println!("Pushing branch '{}' to remote '{}'", branch, remote);
            Ok(())
        }
        BitCommands::Init => phases::init::check_dir(),

        BitCommands::HashObject { path, write } => objects::process_command(path, *write),

        BitCommands::CatFile { hash, print } => objects::cat_object(hash, *print),
    }
}
