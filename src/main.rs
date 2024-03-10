use anyhow::Context;
use clap::Parser;
use clap::Subcommand;
use flate2::bufread::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::Digest;
use sha1::Sha1;
use std::ffi::CStr;
use std::fs;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
  #[command(subcommand)]
  command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
  Init,
  CatFile {
    #[clap(short = 'p')]
    pretty_print: bool,
    object_hash: String,
  },
  HashObject {
    #[clap(short = 'w')]
    write: bool,
    file: PathBuf,
  },
}

enum Kind {
  Blob,
}

fn main() -> anyhow::Result<()> {
  let args = Args::parse();
  // You can use print statements as follows for debugging, they'll be visible when running tests.
  // eprintln!("Logs from your program will appear here!");
  match args.command {
    Command::Init => init()?,
    Command::CatFile {
      pretty_print,
      object_hash,
    } => cat_file(pretty_print, object_hash)?,
    Command::HashObject { write, file } => hash_object(write, file)?,
  }
  Ok(())
}

fn init() -> anyhow::Result<()> {
  fs::create_dir(".git")?;
  fs::create_dir(".git/objects")?;
  fs::create_dir(".git/refs")?;
  fs::write(".git/HEAD", "ref: refs/heads/master\n")?;
  println!("Initialized git directory");
  Ok(())
}

fn hash_object(write: bool, file: PathBuf) -> anyhow::Result<()> {
  let stat = std::fs::metadata(&file).with_context(|| format!("stat {}", file.display()))?;
  let size = format!("{}", stat.len());
  let mut blob = Vec::<u8>::new();
  blob.extend_from_slice(format!("blob {size}\0").as_bytes());
  let mut f = std::fs::File::open(file).context("reading the passed file")?;
  f.read_to_end(&mut blob).context("reading the given file")?;
  let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
  e.write_all(&blob)?;
  let out = e.finish().context("completing the write")?;
  let mut hasher = Sha1::new();
  hasher.update(&out[..]);
  let filehash = hex::encode(hasher.finalize());
  println!("{filehash}");
  if write {
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

fn cat_file(pretty_print: bool, object_hash: String) -> anyhow::Result<()> {
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
