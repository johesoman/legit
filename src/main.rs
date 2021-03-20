use anyhow::*;
use std::env;
use std::fs;
use std::path;
use walkdir::WalkDir;

mod object_database;
use object_database::*;

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

fn is_not_git_entry(entry: &walkdir::DirEntry) -> bool {
  !entry.path().starts_with(get_cwd().join(".git"))
}

fn do_commit() -> Result<()> {
  let walker = WalkDir::new(get_cwd()).into_iter();

  let object_database = ObjectDatabase::new(get_cwd().join(".git/objects"));

  let paths: Vec<_> = walker
    .filter_entry(is_not_git_entry)
    .filter_map(|entry| {
      let entry = entry.unwrap();
      let is_not_dir = !entry.file_type().is_dir();
      is_not_dir.then(move || entry.path().to_path_buf())
    })
    .collect();

  let blobs: Vec<_> = paths
    .iter()
    .map(|path| {
      let contents = std::fs::read(path).unwrap();
      Blob::new(&contents[..])
    })
    .collect();

  let entries = paths.iter().zip(blobs.iter()).map(|(path, blob)| {
    let path = path.strip_prefix(get_cwd()).unwrap().to_path_buf();
    Entry::new(&path, blob.object_id())
  }).collect::<Vec<_>>();

  for blob in blobs {
    object_database.write_object(&blob)?;
  }

  let tree = Tree::new(entries);
  object_database.write_object(&tree)?;

  let commit = Commit {
    author: Contributor::new("Martin Söderman", "kngrektor@gmail.com"),
    authored_at: chrono::offset::Utc::now(),
    committer: Contributor::new("Martin Söderman", "kngrektor@gmail.com"),
    committed_at: chrono::offset::Utc::now(),
    message: "Väldigt coolt meddelanden!".to_owned(),
    tree_object_id: tree.object_id()
  };
  object_database.write_object(&commit)?;

  std::fs::write(get_cwd().join(".git/HEAD"), commit.object_id().as_hex())?;

  println!("[(root-commit) {}] {}", commit.object_id().as_hex(), commit.message);
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
