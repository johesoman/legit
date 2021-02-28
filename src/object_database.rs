use anyhow::*;
use ring::digest::{digest, SHA1_FOR_LEGACY_USE_ONLY};
use std::convert::TryInto;
use std::path;

// +--------+
// | Object |
// +--------+

pub type ObjectId = [u8; 20];

pub trait Object: Sized {
  fn serialize(&self) -> Vec<u8>;
  fn deserialize(bytes: Vec<u8>) -> Result<Self>;

  fn object_id(&self) -> ObjectId {
    let serialized = self.serialize();
    digest(&SHA1_FOR_LEGACY_USE_ONLY, &serialized[..])
      .as_ref()
      .try_into()
      .unwrap()
  }

  fn object_id_as_hex(&self) -> String {
    self
      .object_id()
      .iter()
      .map(|b| format!("{:02x}", b))
      .collect::<Vec<String>>()
      .concat()
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

  pub fn write_object<T: Object>(&self, object: T) -> Result<()> {
    let object_id = object.object_id_as_hex();
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
  pub fn new(path: &path::PathBuf, object_id: &ObjectId) -> Self {
    Entry {
      mode: 100644,
      path: path.to_owned(),
      object_id: object_id.to_owned(),
    }
  }
}

// +------+
// | Tree |
// +------+

pub struct Tree {
  entries: Vec<Entry>,
}

// impl Tree {
//   fn new(entries: Vec<Entry>) -> Self {}
// }
