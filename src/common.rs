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
  Commit,
}
impl Kind {
  fn from_str(kind: &str) -> anyhow::Result<Kind> {
    match kind {
      "blob" => Ok(Kind::Blob),
      "tree" => Ok(Kind::Tree),
      _ => anyhow::bail!("should not be called for: '{kind}'"),
    }
  }
}

impl Display for Kind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Kind::Blob => write!(f, "blob"),
      Kind::Tree => write!(f, "tree"),
      Kind::Commit => write!(f, "commit"),
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
  pub fn hash_bytes(&self) -> anyhow::Result<[u8; 20]> {
    GitObject::_hash(&self.data)
  }

  pub fn hash(&self) -> anyhow::Result<String> {
    let hash = self.hash_bytes()?;
    Ok(hex::encode(hash))
  }

  pub fn write(&self, repo_path: &Path) -> anyhow::Result<()> {
    let hash = self.hash()?;
    let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
    e.write_all(&self.data)?;
    let out = e.finish().context("completing the write")?;
    let dest_dir = repo_path.join(format!(".git/objects/{}", &hash[..2]));
    std::fs::create_dir_all(dest_dir.clone()).context("creating git objects directory")?;
    let dest_file = dest_dir.join(&hash[2..]);
    let mut write = std::fs::File::create(dest_file).context("writing hashed file")?;
    write.write_all(&out)?;
    write.flush()?;
    Ok(())
  }

  pub(crate) fn stdout(&self, writer: &mut dyn Write) -> anyhow::Result<()> {
    let mut reader = BufReader::new(&self.data[..]);
    let mut buf_header = Vec::<u8>::new();
    reader.read_until(0, &mut buf_header)?;
    let header = CStr::from_bytes_with_nul(&buf_header)
      .context("malformed blob object")?
      .to_str()?;
    let header = header.to_string();
    let (kind, size) = header
      .split_once(' ')
      .ok_or(anyhow::anyhow!("malformed blob object"))?;
    let size = size.parse::<u64>().context("malformed blob object")?;
    let kind = Kind::from_str(kind)?;
    let mut reader = reader.take(size);
    match kind {
      Kind::Blob => {
        let n =
          std::io::copy(&mut reader, writer).context("copying from .git/objects to stdout")?;
        anyhow::ensure!(
          n == self.size,
          ".git/object file not of expected size, expected: '{}', actual: '{}'",
          self.size,
          n
        );
      }
      Kind::Tree => {
        let mut buf = Vec::new();
        let mut entries = Vec::new();
        loop {
          buf.clear();
          let n = reader.read_until(0, &mut buf)?;
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
          reader.read_exact(&mut [0u8; 20])?; // ignore sha
        }
        entries
          .into_iter()
          .for_each(|entry| writeln!(writer, "{}", entry).unwrap());
      }
      Kind::Commit => anyhow::bail!("stdout not implemented for commit object"),
    }
    Ok(())
  }
}

impl GitObject {
  pub fn _hash(blob: &[u8]) -> anyhow::Result<[u8; 20]> {
    let mut hasher = Sha1::new();
    hasher.update(blob);
    let hash = hasher.finalize();
    Ok(hash.as_slice().try_into().expect("hash is always 20 bytes"))
  }

  pub fn read_object(repo_path: &Path, object_hash: &str) -> anyhow::Result<GitObject> {
    let filepath = format!(".git/objects/{}/{}", &object_hash[..2], &object_hash[2..]);
    let filepath = repo_path.join(filepath);
    let f = std::fs::File::open(&filepath)
      .with_context(|| format!("opening file {}", &filepath.to_string_lossy()))?;
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

  pub fn build_tree_object(current_path: &Path) -> anyhow::Result<GitObject> {
    if !current_path.is_dir() {
      anyhow::bail!("{} is not a directory", current_path.display())
    }
    let mut result = std::fs::read_dir(current_path)?
      .filter_map(|entry| entry.ok())
      .filter(|entry| {
        let path = entry.path();
        let relative = path.strip_prefix(current_path).unwrap();
        !relative.starts_with(".git")
      })
      .collect::<Vec<_>>();
    result.sort_by_key(|a| a.file_name());
    let result = result
      .into_iter()
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
            GitObject::build_file_object(&entry.path()).unwrap(),
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
        Kind::Tree => "40000",
        Kind::Commit => anyhow::bail!("permission not required for commit object"),
      };
      let relative_path = path.strip_prefix(current_path)?;
      output
        .extend_from_slice(format!("{} {}\0", mode, relative_path.to_string_lossy()).as_bytes());
      output.extend_from_slice(&entry.hash_bytes()?);
    }
    let size = output.len() as u64;
    let header = format!("tree {}\x00", size);
    let mut data = Vec::<u8>::new();
    data.extend_from_slice(header.as_bytes());
    data.extend_from_slice(&output[..]);
    Ok(GitObject {
      kind: Kind::Tree,
      size,
      data,
      _private: (),
    })
  }

  pub fn build_commit_object(
    tree_hash: &str,
    repo_path: &Path,
    message: &str,
    parent: Option<String>,
  ) -> anyhow::Result<GitObject> {
    let tree_object = GitObject::read_object(repo_path, tree_hash)?;
    let mut commit_content = Vec::from(format!("tree {}\n", tree_object.hash()?).as_bytes());
    if let Some(parent) = parent {
      commit_content.extend_from_slice(format!("parent {}\n", parent).as_bytes());
    }
    commit_content.extend_from_slice("author Coder <coder@crafters.io>\n".as_bytes());
    commit_content.extend_from_slice("committer Coder <coder@crafters.io>\n\n".as_bytes());
    commit_content.extend_from_slice(message.as_bytes());
    commit_content.extend_from_slice("\n".as_bytes());
    let size = commit_content.len() as u64;
    let mut data = Vec::from(format!("commit {}\x00", size).as_bytes());
    data.extend_from_slice(&commit_content);
    Ok(GitObject {
      kind: Kind::Commit,
      size,
      data,
      _private: (),
    })
  }
}
