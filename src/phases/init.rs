// Trigger the init routine. First check if there is any .bit folder and if so, dont to anything,
// else, create the .bit dirtory with an index file.
use crate::utils::path;
use std::fs;
use std::io;

const INIT_FILES: [&str; 2] = ["./.bit/index", "./.bit/HEAD"];
const INIT_DIRS: [&str; 3] = ["./.bit", "./.bit/objects", "./.bit/refs/master"];

/// check if repo exists else, inits a bit repo.
/// Auxiliary function ran to make sure we do not overwrite the old contents.
pub fn check_dir() -> io::Result<()> {
    if path::find_bit_dir().is_some() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "There is already a bit repository",
        ));
    }

    init_repo()
}
/// Function that instantiate the bit directory
fn init_repo() -> io::Result<()> {
    for path in INIT_DIRS {
        fs::create_dir_all(path)?;
    }

    for file in INIT_FILES {
        fs::File::create(file)?;
    }

    println!("Bit repo initiated!");
    Ok(())
}
