use std::fs;

pub(crate) fn init() -> anyhow::Result<()> {
  fs::create_dir(".git")?;
  fs::create_dir(".git/objects")?;
  fs::create_dir(".git/refs")?;
  fs::write(".git/HEAD", "ref: refs/heads/master\n")?;
  println!("Initialized git directory");
  Ok(())
}