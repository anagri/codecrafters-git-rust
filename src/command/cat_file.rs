use anyhow::Context;
use flate2::bufread::ZlibDecoder;
use std::ffi::CStr;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;

use crate::command::common::Kind;

pub(crate) fn cat_file(pretty_print: bool, object_hash: String) -> anyhow::Result<()> {
  anyhow::ensure!(pretty_print, "only supports pretty print");
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
  let header = CStr::from_bytes_with_nul(&buf).expect("expecting valid c-string ending with '\\0'");
  let header = header
    .to_str()
    .context("git/objects file header isn't valid UTF-8")?;
  let Some((kind, size)) = header.split_once(' ') else {
    anyhow::bail!(".git/objects file header did not start with a known type: '{header}'");
  };
  let kind = match kind {
    "blob" => Kind::Blob,
    _ => anyhow::bail!("don't support kind: '{kind}'"),
  };
  let size = size
    .parse::<u64>()
    .context(".git/objects file header has invalid size: {size}")?;
  let mut z = z.take(size);
  match kind {
    Kind::Blob => {
      let stdout = std::io::stdout();
      let mut stdout = stdout.lock();
      let n = std::io::copy(&mut z, &mut stdout).context("copying from .git/objects to stdout")?;
      anyhow::ensure!(
        n == size,
        ".git/object file not of expected size, expected: '{size}', actual: '{n}'"
      );
    }
  }
  Ok(())
}
