use anyhow::*;
use path::Display;
use ring::digest::{digest, SHA1_FOR_LEGACY_USE_ONLY};
use std::convert::{Into, TryInto};
use std::path;
use chrono::prelude::*;

// +--------+
// | Object |
// +--------+
#[derive(Clone, Copy)]
pub struct ObjectId([u8; 20]);

impl ObjectId {
  pub fn as_bytes(&self) -> [u8; 20] {
    self.0
  }

  pub fn as_hex(&self) -> String {
    self.0.iter()
      .map(|b| format!("{:02x}", b))
      .collect::<String>()
  }
}

impl std::fmt::Display for ObjectId {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.as_hex())
  }
}

pub trait Object: Sized {
  fn serialize(&self) -> Vec<u8>;
  fn deserialize(bytes: Vec<u8>) -> Result<Self>;

  fn object_id(&self) -> ObjectId {
    let serialized = self.serialize();
    let object_id = digest(&SHA1_FOR_LEGACY_USE_ONLY, &serialized[..])
      .as_ref()
      .try_into()
      .unwrap();

    ObjectId(object_id)
  }
}

// +----------------+
// | ObjectDatabase |
// +----------------+

pub struct ObjectDatabase {
  objects_path: path::PathBuf,
}

impl ObjectDatabase {
  pub fn new<T: Into<path::PathBuf>>(objects_path: T) -> Self {
    ObjectDatabase {
      objects_path: objects_path.into(),
    }
  }

  pub fn write_object<T: Object>(&self, object: &T) -> Result<()> {
    let object_id = object.object_id().as_hex();
    let (dir_name, file_name) = object_id.split_at(2);

    let dir_name = self.objects_path.join(dir_name);

    std::fs::create_dir_all(&dir_name)?;
    std::fs::write(&dir_name.join(file_name), object.serialize())?;

    Ok(())
  }
}

// +------+
// | Blob |
// +------+

pub struct Blob {
  pub contents: Vec<u8>,
}

impl Blob {
  pub fn new(contents: &[u8]) -> Self {
    Blob {
      contents: contents.to_vec(),
    }
  }
}

impl Object for Blob {
  fn serialize(&self) -> Vec<u8> {
    let header = format!("blob {}\0", self.contents.len());
    let mut contents: Vec<u8> = header.into_bytes();
    contents.extend(&self.contents[..]);
    contents
  }

  fn deserialize(bytes: Vec<u8>) -> Result<Blob> {
    if let Some(bytes) = bytes.split(|b| *b == 0u8).nth(1) {
      Ok(Blob::new(&bytes.to_vec()))
    } else {
      Err(anyhow!("ERROR"))
    }
  }
}

// +-------+
// | Entry +
// +-------+

pub struct Entry {
  pub mode: u64,
  pub path: path::PathBuf,
  pub object_id: ObjectId,
}

impl Entry {
  pub fn new(path: &path::PathBuf, object_id: ObjectId) -> Self {
    Entry {
      mode: 100644,
      path: path.to_owned(),
      object_id: object_id,
    }
  }
}

// +------+
// | Tree |
// +------+

pub struct Tree {
  entries: Vec<Entry>,
}

impl Tree {
  pub fn new(entries: Vec<Entry>) -> Self {
    Tree { entries }
  }
}

impl Object for Tree {
  fn serialize(&self) -> Vec<u8> {
    let mut bytes = self.entries.iter()
      .map(|entry| {
        let mut bytes = format!("{} {}\0", entry.mode, entry.path.display()).into_bytes();
        bytes.extend(&entry.object_id.as_bytes());
        bytes
      })
      .flatten()
      .collect::<Vec<_>>();
    
    let mut contents = format!("tree {}\0", bytes.len()).into_bytes();
    contents.append(&mut bytes);
    contents
  }

  fn deserialize(bytes: Vec<u8>) -> Result<Tree> {
    std::unimplemented!()
  }
}

// +-------------+
// | Contributor |
// +-------------+

pub struct Contributor {
  pub name: String,
  pub email: String
}

impl Contributor {
  pub fn new<T: Into<String>, U: Into<String>>(name: T, email: U) -> Contributor {
    Contributor {
      name: name.into(),
      email: email.into()
    }
  }
}

impl std::fmt::Display for Contributor {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{} <{}>", self.name, self.email)
  }
}

// +--------+
// | Commit |
// +--------+

pub struct Commit {
  pub author: Contributor,
  pub authored_at: chrono::DateTime<Utc>,
  pub committer: Contributor,
  pub committed_at: chrono::DateTime<Utc>,
  pub message: String,
  pub tree_object_id: ObjectId
}

impl Commit {
  pub fn new(
    author: Contributor,
    authored_at: chrono::DateTime<Utc>,
    committer: Contributor,
    committed_at: chrono::DateTime<Utc>,
    message: String,
    tree_object_id: ObjectId
  ) -> Commit {
    Commit {
      author, authored_at,
      committer, committed_at,
      message,
      tree_object_id
    }
  }
  pub fn message_summary(&self) -> &str {
    let len = self.message.find('\n')
      .unwrap_or(40)
      .min(40);
    &self.message[..len]
  }
}

impl Object for Commit {
  fn serialize(&self) -> Vec<u8> {
  let mut bytes = format!(r#"tree {}
      author {} {}
      commit {} {}
      {}"#,
      self.tree_object_id,
      self.author, self.authored_at.format("%s %z"),
      self.committer, self.committed_at.format("%s %z"),
      self.message
    ).into_bytes();
    
    let mut contents = format!("commit {}\0", bytes.len()).into_bytes();
    contents.append(&mut bytes);
    contents
  }

  fn deserialize(bytes: Vec<u8>) -> Result<Commit> {
    std::unimplemented!()
  }
}