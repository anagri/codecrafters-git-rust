use clap::Parser;
use clap::Subcommand;
#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)]
use std::fs;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
  #[command(subcommand)]
  command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
  Init,
}

fn main() {
  let args = Args::parse();
  // You can use print statements as follows for debugging, they'll be visible when running tests.
  println!("Logs from your program will appear here!");
  match args.command {
    Command::Init => {
      fs::create_dir(".git").unwrap();
      fs::create_dir(".git/objects").unwrap();
      fs::create_dir(".git/refs").unwrap();
      fs::write(".git/HEAD", "ref: refs/heads/master\n").unwrap();
      println!("Initialized git directory");
    }
  }
}
