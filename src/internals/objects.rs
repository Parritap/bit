use const_hex::{self, encode};
use flate2::Compression;
use flate2::write::GzEncoder;
use sha1::{Digest, Sha1};
use std::io::{Result, Write};
use std::vec::Vec;
use std::{
    fs::{File, Metadata},
    io::{Read, copy},
    os::unix::fs::MetadataExt,
};
use strum_macros::{Display, IntoStaticStr};

/// Function that creates an object give the path of a file.
/// It determines whether the file is regular or a directory.
///

///
#[derive(Display, IntoStaticStr)]
enum FileType {
    #[strum(serialize = "tree")]
    Dir,
    #[strum(serialize = "blob")]
    Regular,
}

//TODO: Deal with directory case.
pub fn create(path: &str) -> Result<()> {
    //Determine whether it is a tree or a blob.

    let metadata: Metadata = std::fs::metadata(path)?;

    if metadata.is_dir() {
        create_dir_obj(path, metadata)
    } else {
        create_blob_obj(path, metadata)
    }
}

fn create_blob_obj(path: &str, metadata: Metadata) -> Result<()> {
    // Computate the hash based on the header and its the reg-file's content
    let size = metadata.size();
    let mut content_buf: Vec<u8> = Vec::new();
    let mut target_file = File::open(path)?;
    target_file.read_to_end(&mut content_buf)?; //Writes the content of file into buf.
    //in the line below, the FileType enum gets converted into a string at compile time thanks to strum_macros crate
    let mut object_data: Vec<u8> = format!("{} {}\0", FileType::Regular, size)
        .as_bytes()
        .to_vec();
    object_data.append(&mut content_buf);

    let mut hasher = Sha1::new();
    hasher.update(&object_data);
    let hash_array: [u8; 20] = hasher.finalize().into();
    let mut hex_hash = const_hex::encode(hash_array);

    println!("Printing object hex_hash: {}", hex_hash);

    // Now lets check if a dir with that hash is already save in the database
    let tail: String = hex_hash.split_off(2);
    println!("{hex_hash}, {tail}");
    std::fs::create_dir_all(format!("./.bit/objects/{hex_hash}"))?;
    let mut obj_file = File::create(format!("./.bit/objects/{hex_hash}/{tail}"))?;

    let mut encoder = GzEncoder::new(&mut obj_file, Compression::default());
    println!("Creating binary object");
    encoder.write_all(content_buf.as_slice())?;

    // Now lets save the contents of in a compressed form inside said file.
    Ok(())
}

fn create_dir_obj(path: &str, metadata: Metadata) -> Result<()> {
    //
    Ok(())
}
