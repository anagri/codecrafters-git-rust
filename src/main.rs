mod command;
use clap::Parser;
use clap::Subcommand;
use command::cat_file;
use command::hash_object;
use command::init;
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
