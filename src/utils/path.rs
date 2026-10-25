use std::path::PathBuf;

pub fn find_bit_dir() -> Option<PathBuf> {
    // Walk up (higher up) from the current directory until a .bit folder is found.
    let mut current = std::env::current_dir().ok()?;

    loop {
        let candidate = current.join(".bit");
        if candidate.is_dir() {
            return Some(candidate);
        }

        //This pop allow us to go even until the system root folder.
        if !current.pop() {
            return None;
        }
    }
}
