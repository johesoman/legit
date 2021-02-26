use anyhow::*;
use ring::digest;
use std::env;
use std::fs;
use std::path;
use walkdir::WalkDir;

fn get_cwd() -> path::PathBuf {
  path::Path::new("test/repo").to_path_buf()
}

fn do_init() -> Result<()> {
  let dirs = vec![".git", ".git/objects", ".git/refs"];

  for dir in dirs {
    fs::create_dir_all(get_cwd().join(dir))?;
  }

  println!("Repository initialized!");
  Ok(())
}

struct Blob {
  pub bytes: Vec<u8>,
  pub object_id: Vec<u8>,
}

impl Blob {
  fn new(bytes: &[u8]) -> Blob {
    let object_id = digest::digest(&digest::SHA1_FOR_LEGACY_USE_ONLY, bytes)
      .as_ref()
      .to_vec();

    Blob {
      object_id,
      bytes: bytes.to_vec(),
    }
  }

  fn serialize(&self) -> Vec<u8> {
    let header = format!("blob {}\0", self.bytes.len());
    let mut bytes: Vec<u8> = header.into_bytes();
    bytes.extend(&self.bytes[..]);
    bytes
  }

  fn deserialize(bytes: Vec<u8>) -> Result<Blob> {
    if let Some(bytes) = bytes.split(|b| *b == 0u8).nth(1) {
      Ok(Blob::new(&bytes.to_vec()))
    } else {
      Err(anyhow!("ERROR"))
    }
  }
}

fn is_not_git_entry(entry: &walkdir::DirEntry) -> bool {
  !entry.path().starts_with(get_cwd().join(".git"))
}

fn do_commit() -> Result<()> {
  let walker = WalkDir::new(get_cwd()).into_iter();
  for entry in walker.filter_entry(is_not_git_entry) {
    let entry = entry?;

    if !entry.file_type().is_dir() {
      println!("{:?}", entry.path());
    }
  }

  println!("You made a commit!");
  Ok(())
}

fn do_help() -> Result<()> {
  println!("Here's the help!");
  Ok(())
}

fn main() -> Result<()> {
  let mut args = env::args().skip(1);

  let command = args.next().unwrap_or("help".to_owned());

  match command.as_str() {
    "init" => do_init()?,
    "commit" => do_commit()?,
    "help" => do_help()?,
    _ => println!("Error: unknown command {:?}!", command),
  }

  Ok(())
}
