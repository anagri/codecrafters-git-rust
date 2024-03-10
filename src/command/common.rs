use anyhow::Context;
use flate2::bufread::ZlibDecoder;
use std::ffi::CStr;
use std::fmt::Display;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Kind {
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

pub(crate) struct GitObject<R> {
  pub(crate) kind: Kind,
  pub(crate) size: u64,
  pub(crate) reader: R,
}
impl<R> GitObject<R>
where
  R: BufRead,
{
  pub(crate) fn stdout(&mut self) -> anyhow::Result<()> {
    match self.kind {
      Kind::Blob => todo!(),
      Kind::Tree => self.tree()?,
    }
    Ok(())
  }

  fn tree(&mut self) -> anyhow::Result<()> {
    let mut entries = Vec::new();
    let mut buf = Vec::new();
    loop {
      buf.clear();
      let n = self.reader.read_until(0, &mut buf)?;
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
      self.reader.read_exact(&mut [0u8; 20])?; // ignore sha
    }
    entries.into_iter().for_each(|entry| println!("{}", entry));
    Ok(())
  }
}

impl GitObject<()> {
  pub(crate) fn read_object(object_hash: String) -> anyhow::Result<GitObject<impl BufRead>> {
    let f = std::fs::File::open(format!(
      ".git/objects/{}/{}",
      &object_hash[..2],
      &object_hash[2..]
    ))
    .context("open in .git/objects")?;
    let f = BufReader::new(f);
    let z = ZlibDecoder::new(f);
    let mut z = BufReader::new(z);
    let mut buf = Vec::<u8>::new();
    z.read_until(0, &mut buf)
      .context("read header from .git/objects")?;
    let header =
      CStr::from_bytes_with_nul(&buf).expect("expecting valid c-string ending with '\\0'");
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
    let z = z.take(size);
    Ok(GitObject {
      kind,
      size,
      reader: z,
    })
  }
}
