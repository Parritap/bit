use const_hex;
use sha1::{Digest, Sha1};
use std::vec::Vec;
use std::{
    fs::{File, Metadata},
    io::Read,
    os::unix::fs::MetadataExt,
};
use strum_macros::{Display, IntoStaticStr};

/// Function that creates an object give the path of a file.
/// It determines whether the file is regular or a directory.
///

#[derive(Display, IntoStaticStr)]
enum FileType {
    #[strum(serialize = "tree")]
    Dir,
    #[strum(serialize = "blob")]
    Regular,
}

//TODO: Deal with directory case.
pub fn create(path: &str) -> std::io::Result<()> {
    //Determine whether it is a tree or a blob.

    let metadata: Metadata = std::fs::metadata(path)?;
    let dir_type: FileType = if metadata.is_dir() {
        FileType::Dir
    } else {
        FileType::Regular
    };
    let size = metadata.size();
    let mut content_buf: Vec<u8> = Vec::new();
    File::open(path)?.read_to_end(&mut content_buf)?; //Writes the content of file into buf.

    let mut object_data: Vec<u8> = format!("{} {}\0", dir_type, size).as_bytes().to_vec(); //dir_type is converted into
    //string thanks to the strum_macros crate.

    object_data.append(&mut content_buf);

    let mut hasher = Sha1::new();
    hasher.update(&object_data);
    let hash_array: [u8; 20] = hasher.finalize().into();
    let hex_hash = const_hex::encode(hash_array);

    println!("Printing object hex_hash: {}", hex_hash);

    Ok(())
}
