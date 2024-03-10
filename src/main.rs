mod command;
use crate::command::common::Kind;
use crate::command::GitObject;
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
  LsTree {
    #[clap(long)]
    name_only: bool,
    object_hash: String,
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
    Command::LsTree {
      name_only,
      object_hash,
    } => ls_tree(name_only, object_hash)?,
  }
  Ok(())
}

fn ls_tree(name_only: bool, object_hash: String) -> anyhow::Result<()> {
  anyhow::ensure!(name_only, "only --name-only is supported");
  let mut git_object = GitObject::read_object(object_hash)?;
  if git_object.kind != Kind::Tree {
    Err(anyhow::anyhow!("fatal: not a tree object"))?;
  }
  git_object.stdout()?;
  Ok(())
}
