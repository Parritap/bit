use crate::utils::path;
use const_hex;
use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use sha1::{Digest, Sha1};
use std::fmt::format;
use std::fs;
use std::io::{Error, ErrorKind, Result, Write};
use std::ptr::null;
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
    let mut header_data: Vec<u8> = format!("{} {}\0", FileType::Regular, size)
        .as_bytes()
        .to_vec();
    header_data.append(&mut content_buf);

    let mut hasher = Sha1::new();
    hasher.update(&header_data);
    let hash_array: [u8; 20] = hasher.finalize().into();
    let mut hex_hash = const_hex::encode(hash_array);

    println!("Printing object hex_hash: {}", hex_hash);

    // Now lets check if a dir with that hash is already save in the database
    let tail: String = hex_hash.split_off(2);
    println!("{hex_hash}, {tail}");
    let bit_dir = path::find_bit_dir().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "could not find .bit directory",
        )
    })?;
    let object_dir = bit_dir.join("objects").join(&hex_hash);

    std::fs::create_dir_all(&object_dir)?;
    let mut obj_file = File::create(object_dir.join(&tail))?;

    let mut encoder = GzEncoder::new(&mut obj_file, Compression::default());
    println!("Creating binary object");
    encoder.write_all(header_data.as_slice())?;
    encoder.finish()?;

    // Now lets save the contents of in a compressed form inside said file.
    Ok(())
}

fn create_dir_obj(_path: &str, _metadata: Metadata) -> Result<()> {
    // TODO:
    let _ = FileType::Dir;
    Ok(())
}

pub fn cat_object(hash_str: &str) -> Result<()> {
    // Check first if a bit directory exists
    let bit_dir = path::find_bit_dir()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "Bit repo not found"))?;

    let bit_str = bit_dir.to_str().unwrap();
    println!("{bit_str}");
    let (head, tail) = hash_str.split_at(2);
    //println!("head:{head}\ntail:{tail}");
    //
    // Declare the file that will hold the value of the hash so we can uncompress it

    let obj_file = format!("{bit_str}/objects/{head}/{tail}");
    println!("{obj_file}");

    if fs::exists(&obj_file).is_ok() {
        print_encoded_file(obj_file.as_str())
    } else {
        Err(Error::new(
            ErrorKind::NotFound,
            "The specified object was not found",
        ))
    }
}

fn print_encoded_file(path: &str) -> Result<()> {
    let obj_file = File::open(path).unwrap();
    let mut decoder = GzDecoder::new(obj_file);
    let mut contents = String::new();
    decoder.read_to_string(&mut contents)?;

    let null_index = contents.as_bytes().iter().position(|b| *b == 0).unwrap();

    let body = contents.split_at(null_index).1;
    println!("{body}");

    Ok(())
}
