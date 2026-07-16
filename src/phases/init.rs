// Trigger the init routine. First check if there is any .bit folder and if so, dont to anything,
// else, create the .bit dirtory with an index file.
use std::fs;
use std::fs::exists;
use std::process::ExitCode;

const INIT_FILES: [&str; 2] = ["./.bit/index", "./.bit/HEAD"];
const INIT_DIRS: [&str; 3] = ["./.bit", "./.bit/objects", "./.bit/refs/master"];

pub fn check_dir() -> ExitCode {
    match exists("./.bit") {
        Ok(true) => {
            println!("There is already a bit repository");
            ExitCode::FAILURE
        }
        Ok(false) => init_repo(),
        Err(m) => {
            println!("{m}");
            ExitCode::FAILURE
        }
    }
}
/// Function that instantiate the bit directory
fn init_repo() -> ExitCode {
    let mut dir_builder = fs::DirBuilder::new();
    dir_builder.recursive(true);

    for path in INIT_DIRS {
        match dir_builder.create(path) {
            Ok(_p) => {}
            Err(message) => println!("{message}"),
        }
    }

    for file in INIT_FILES {
        fs::File::create(file).expect("Error creating file: {file}");
    }
    println!("Bit repo initiated!");
    ExitCode::SUCCESS
}
