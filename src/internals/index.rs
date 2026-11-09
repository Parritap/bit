use file_mode::{ModePath, User};
use std::io::Error;
use std::path::Path;

pub fn populate_index() -> Result<(), Error> {
    print_mode("./Cargo.toml")?;
    Ok(())
}

//TODO: Print each bit of the mode of the file in a printable format.
fn print_mode(path: &str) -> Result<(), std::io::Error> {
    let file_mode = Path::new(path).mode()?;
    println!(file_mode.into());
    Ok(())
}
