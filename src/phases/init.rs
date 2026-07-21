// Trigger the init routine. First check if there is any .bit folder and if so, dont to anything,
// else, create the .bit dirtory with an index file.
use crate::utils::path;
use std::fs::exists;
use std::path::PathBuf;
use std::process::ExitCode;
use std::{any, fs};

const INIT_FILES: [&str; 2] = ["./.bit/index", "./.bit/HEAD"];
const INIT_DIRS: [&str; 3] = ["./.bit", "./.bit/objects", "./.bit/refs/master"];

/// check if repo exists else, inits a bit repo.
/// Auxiliary function ran to make sure we do not overwrite the old contents.
pub fn check_dir() -> Result<(), &'static str> {
    let bit_repo = path::find_bit_dir();

    return if let Some(_) = bit_repo {
        Err("There is already a bit repository")
    } else {
        init_repo()
    };
}
/// Function that instantiate the bit directory
fn init_repo() -> Result<(), &str> {
    let mut dir_builder = fs::DirBuilder::new();
    dir_builder.recursive(true);

    for path in INIT_DIRS {
        dir_builder.create(path);
    }
    for file in INIT_FILES {
        fs::File::create(file);
    }
    println!("Bit repo initiated!");
    Ok(())
}
