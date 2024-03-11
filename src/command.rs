use crate::common::build_file_object;
use crate::common::build_tree_object;
use crate::common::GitObject;
use crate::common::Kind;
use std::fs;
use std::io;
use std::path::Path;

pub fn init(path: &Path, writer: &mut dyn io::Write) -> anyhow::Result<()> {
  fs::create_dir(path.join(".git"))?;
  fs::create_dir(path.join(".git/objects"))?;
  fs::create_dir(path.join(".git/refs"))?;
  fs::write(path.join(".git/HEAD"), "ref: refs/heads/master\n")?;
  write!(writer, "Initialized git directory")?;
  Ok(())
}

pub fn hash_object(path: &Path, writer: &mut dyn io::Write, write: bool) -> anyhow::Result<()> {
  let git_object = build_file_object(path)?;
  let hash = git_object.hash()?;
  writeln!(writer, "{hash}")?;
  if write {
    git_object.write()?;
  }
  Ok(())
}

pub fn cat_file(pretty_print: bool, object_hash: String) -> anyhow::Result<()> {
  anyhow::ensure!(pretty_print, "only supports pretty print");
  let mut git_object = GitObject::read_object(object_hash)?;
  git_object.stdout()?;
  Ok(())
}

pub fn write_tree(path: &Path) -> anyhow::Result<()> {
  let tree_object = build_tree_object(path)?;
  tree_object.write()?;
  println!("{}", tree_object.hash()?);
  Ok(())
}

pub fn ls_tree(name_only: bool, object_hash: String) -> anyhow::Result<()> {
  anyhow::ensure!(name_only, "only --name-only is supported");
  let mut git_object = GitObject::read_object(object_hash)?;
  if git_object.kind != Kind::Tree {
    Err(anyhow::anyhow!("fatal: not a tree object"))?;
  }
  git_object.stdout()?;
  Ok(())
}
