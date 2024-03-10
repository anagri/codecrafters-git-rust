use std::{
  io::{Read, Write},
  path::PathBuf,
};

use anyhow::Context;
use flate2::{write::ZlibEncoder, Compression};
use sha1::{Digest, Sha1};

pub(crate) fn hash_object(write: bool, file: PathBuf) -> anyhow::Result<()> {
  let stat = std::fs::metadata(&file).with_context(|| format!("stat {}", file.display()))?;
  let size = format!("{}", stat.len());
  let mut blob = Vec::<u8>::new();
  blob.extend_from_slice(format!("blob {size}\0").as_bytes());
  let mut f = std::fs::File::open(file).context("reading the passed file")?;
  f.read_to_end(&mut blob).context("reading the given file")?;
  let mut hasher = Sha1::new();
  hasher.update(&blob[..]);
  let filehash = hex::encode(hasher.finalize());
  println!("{filehash}");
  if write {
    let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
    e.write_all(&blob)?;
    let out = e.finish().context("completing the write")?;
    std::fs::create_dir_all(format!(".git/objects/{}", &filehash[..2]))?;
    let mut write = std::fs::File::create(format!(
      ".git/objects/{}/{}",
      &filehash[..2],
      &filehash[2..]
    ))
    .context("writing hashed file")?;
    write.write_all(&out[..])?;
    write.flush()?;
  }
  Ok(())
}
