use anyhow::Context;
use flate2::bufread::ZlibDecoder;
use flate2::{write::ZlibEncoder, Compression};
use sha1::Digest;
use sha1::Sha1;
use std::ffi::CStr;
use std::fmt::Display;
use std::io::Read;
use std::io::Write;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, PartialEq, Eq)]
pub enum Kind {
  Blob,
  Tree,
}

impl Display for Kind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Kind::Blob => write!(f, "blob"),
      Kind::Tree => write!(f, "tree"),
    }
  }
}

#[allow(clippy::manual_non_exhaustive)]
pub struct GitObject {
  pub kind: Kind,
  pub size: u64,
  pub data: Vec<u8>,
  _private: (),
}

impl GitObject {
  fn blob(&self) -> anyhow::Result<Vec<u8>> {
    let mut buf = Vec::<u8>::new();
    buf.extend_from_slice(&self.data[..]);
    Ok(buf)
  }

  pub fn hash(&self) -> anyhow::Result<String> {
    GitObject::_hash(&self.blob()?)
  }

  pub(crate) fn write(&self, repo_path: &Path) -> anyhow::Result<()> {
    let blob: Vec<u8> = self.blob()?;
    let hash = GitObject::_hash(&blob)?;
    let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
    e.write_all(&blob)?;
    let out = e.finish().context("completing the write")?;
    let dest_dir = repo_path.join(format!(".git/objects/{}", &hash[..2]));
    std::fs::create_dir_all(dest_dir.clone()).context("creating git objects directory")?;
    let dest_file = dest_dir.join(&hash[2..]);
    let mut write = std::fs::File::create(dest_file).context("writing hashed file")?;
    write.write_all(&out[..])?;
    write.flush()?;
    Ok(())
  }
  pub(crate) fn stdout(&mut self, writer: &mut dyn Write) -> anyhow::Result<()> {
    match self.kind {
      Kind::Blob => self.stdout_blob(writer)?,
      Kind::Tree => self.stdout_tree(writer)?,
    }
    Ok(())
  }

  fn stdout_tree(&mut self, writer: &mut dyn Write) -> anyhow::Result<()> {
    let mut entries = Vec::new();
    let mut buf_data = BufReader::new(&self.data[..]);
    let mut buf = Vec::new();
    loop {
      buf.clear();
      let n = buf_data.read_until(0, &mut buf)?;
      if n == 0 {
        break;
      }
      let line = CStr::from_bytes_with_nul(&buf)
        .context("malformed tree object")?
        .to_str()?;
      let line = line.to_string();
      let (_mode, name) = line
        .split_once(' ')
        .ok_or(anyhow::anyhow!("malformed tree object"))?;
      entries.push(name.to_string());
      buf_data.read_exact(&mut [0u8; 20])?; // ignore sha
    }
    entries
      .into_iter()
      .for_each(|entry| writeln!(writer, "{}", entry).unwrap());
    Ok(())
  }

  fn stdout_blob(&self, writer: &mut dyn Write) -> anyhow::Result<()> {
    let data = &self.data[..];
    let mut reader = BufReader::new(data);
    let mut buf_header = Vec::<u8>::new();
    reader.read_until(0, &mut buf_header)?;
    let header = CStr::from_bytes_with_nul(&buf_header)
      .context("malformed blob object")?
      .to_str()?;
    let header = header.to_string();
    let (_kind, size) = header
      .split_once(' ')
      .ok_or(anyhow::anyhow!("malformed blob object"))?;
    let size = size.parse::<u64>().context("malformed blob object")?;
    let mut reader = reader.take(size);
    let n = std::io::copy(&mut reader, writer).context("copying from .git/objects to stdout")?;
    anyhow::ensure!(
      n == self.size,
      ".git/object file not of expected size, expected: '{}', actual: '{}'",
      self.size,
      n
    );
    Ok(())
  }
}

impl GitObject {
  fn _hash(blob: &Vec<u8>) -> anyhow::Result<String> {
    let mut hasher = Sha1::new();
    hasher.update(blob);
    let hash = hasher.finalize();
    Ok(hex::encode(hash))
  }

  pub fn read_object(object_hash: &str) -> anyhow::Result<GitObject> {
    let filepath = format!(".git/objects/{}/{}", &object_hash[..2], &object_hash[2..]);
    let f = std::fs::File::open(filepath).context("open in .git/objects")?;
    let f = BufReader::new(f);
    let z = ZlibDecoder::new(f);
    let mut z = BufReader::new(z);
    let mut buf = Vec::<u8>::new();
    z.read_until(0, &mut buf)
      .context("read header from .git/objects")?;
    let header =
      CStr::from_bytes_with_nul(&buf[..]).expect("expecting valid c-string ending with '\\0'");
    let header = header
      .to_str()
      .context("git/objects file header isn't valid UTF-8")?;
    let Some((kind, size)) = header.split_once(' ') else {
      anyhow::bail!(".git/objects file header did not start with a known type: '{header}'");
    };
    let kind = match kind {
      "blob" => Kind::Blob,
      "tree" => Kind::Tree,
      _ => anyhow::bail!("don't support kind: '{kind}'"),
    };
    let size = size
      .parse::<u64>()
      .context(".git/objects file header has invalid size: {size}")?;
    let mut z = z.take(size);
    z.read_to_end(&mut buf)
      .context("reading from .git/objects to end")?;
    Ok(GitObject {
      kind,
      data: buf,
      size,
      _private: (),
    })
  }

  pub fn build_file_object(file: &Path) -> anyhow::Result<GitObject> {
    if !file.is_file() {
      anyhow::bail!("{} is not a file", file.display());
    }
    let stat = std::fs::metadata(file).with_context(|| format!("stat {}", file.display()))?;
    let size = format!("{}", stat.len());
    let mut data = Vec::<u8>::new();
    data.extend_from_slice(format!("{} {}\0", Kind::Blob, size).as_bytes());
    let mut f = std::fs::File::open(file).context("reading the passed file")?;
    f.read_to_end(&mut data).context("reading the given file")?;
    Ok(GitObject {
      kind: Kind::Blob,
      size: stat.len(),
      data,
      _private: (),
    })
  }

  pub fn build_tree_object(path: &Path) -> anyhow::Result<GitObject> {
    if !path.is_dir() {
      anyhow::bail!("{} is not a directory", path.display())
    }
    let result = std::fs::read_dir(path)?
      .filter_map(|entry| entry.ok())
      .filter(|entry| !entry.path().starts_with(".git"))
      .map(|entry| {
        let file_type = entry.file_type().expect("filetype invalid");
        if file_type.is_dir() {
          (
            entry.path(),
            GitObject::build_tree_object(&entry.path()).unwrap(),
          )
        } else if file_type.is_file() {
          (
            entry.path(),
            GitObject::build_file_object(entry.path().as_path()).unwrap(),
          )
        } else {
          panic!("unsupported file type {file_type:?} for {entry:?}")
        }
      })
      .collect::<Vec<_>>();
    let mut output = Vec::<u8>::new();
    for (path, entry) in result {
      let mode = match entry.kind {
        Kind::Blob => "100644",
        Kind::Tree => "040000",
      };
      output.extend_from_slice(
        format!(
          "{} {} {} {}\0",
          mode,
          entry.kind,
          entry.hash()?,
          path.to_string_lossy()
        )
        .as_bytes(),
      );
    }
    let size = output.len() as u64;
    let header = format!("tree {}\0", size);
    output.splice(0..0, header.as_bytes().iter().cloned());
    Ok(GitObject {
      kind: Kind::Tree,
      size,
      data: output,
      _private: (),
    })
  }
}
