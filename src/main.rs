use clap::Parser;
use clap::Subcommand;
use git_starter_rust::command::{cat_file, hash_object, init, ls_tree, write_tree};
use std::env;
use std::io::stdout;
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
  WriteTree,
}

fn main() -> anyhow::Result<()> {
  let args = Args::parse();
  let current_dir = env::current_dir()?;
  let mut stdout = stdout();
  match args.command {
    Command::Init => init(&current_dir, &mut stdout)?,
    Command::CatFile {
      pretty_print,
      object_hash,
    } => cat_file(&object_hash, &mut stdout, &current_dir, pretty_print)?,
    Command::HashObject { write, file } => {
      hash_object(file.as_path(), &mut stdout, &current_dir, write)?
    }
    Command::LsTree {
      name_only,
      object_hash,
    } => ls_tree(&object_hash, &mut stdout, &current_dir, name_only)?,
    Command::WriteTree => write_tree(&current_dir, &mut stdout)?,
  }
  Ok(())
}
