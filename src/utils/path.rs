use std::path::PathBuf;

pub fn find_bit_dir() -> Option<PathBuf> {
    // Walk up from the current directory until a .bit folder is found.
    let mut current = std::env::current_dir().ok()?;

    loop {
        let candidate = current.join(".bit");
        if candidate.is_dir() {
            return Some(candidate);
        }

        if !current.pop() {
            return None;
        }
    }
}
