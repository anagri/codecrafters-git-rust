use anyhow::Context;
use clap::Parser;
use clap::Subcommand;
use flate2::bufread::ZlibDecoder;
#[allow(unused_imports)]
use std::env;
use std::ffi::CStr;
#[allow(unused_imports)]
use std::fs;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;

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
}

fn main() -> anyhow::Result<()> {
  let args = Args::parse();
  // You can use print statements as follows for debugging, they'll be visible when running tests.
  eprintln!("Logs from your program will appear here!");
  match args.command {
    Command::Init => {
      fs::create_dir(".git")?;
      fs::create_dir(".git/objects")?;
      fs::create_dir(".git/refs")?;
      fs::write(".git/HEAD", "ref: refs/heads/master\n")?;
      println!("Initialized git directory");
    }
    Command::CatFile {
      pretty_print,
      object_hash,
    } => {
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
      let header =
        CStr::from_bytes_with_nul(&buf).expect("expecting valid c-string ending with '\\0'");
      let header = header
        .to_str()
        .context("git/objects file header isn't valid UTF-8")?;
      let Some((kind, size)) = header.split_once(' ') else {
        anyhow::bail!(".git/objects file header did not start with a known type: '{header}'");
      };
      if kind != "blob" {
        anyhow::bail!("only supports reading of blob and not {kind}")
      }
      let size = size
        .parse::<usize>()
        .context(".git/objects file header has invalid size: {size}")?;
      buf.clear();
      buf.resize(size, 0);
      z.read_exact(&mut buf[..])
        .context("read contents of .git/objects")?;
      let n = z
        .read(&mut [0])
        .context("validate EOF in .git/object file")?;
      anyhow::ensure!(n == 0, ".git/object file had {n} trailing bytes");
      let stdout = std::io::stdout();
      let mut stdout = stdout.lock();
      stdout
        .write_all(&buf)
        .context("write object contents to stdout")?;
    }
  }
  Ok(())
}
