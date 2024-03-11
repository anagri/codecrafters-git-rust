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

pub fn hash_object(
  path: &Path,
  stdout: &mut dyn io::Write,
  repo: &Path,
  write: bool,
) -> anyhow::Result<()> {
  let git_object = GitObject::build_file_object(path)?;
  let hash = git_object.hash()?;
  writeln!(stdout, "{hash}")?;
  if write {
    git_object.write(repo)?;
  }
  Ok(())
}

pub fn cat_file(
  object_hash: &str,
  writer: &mut dyn io::Write,
  repo_path: &Path,
  pretty_print: bool,
) -> anyhow::Result<()> {
  anyhow::ensure!(pretty_print, "only supports pretty print");
  let git_object = GitObject::read_object(repo_path, object_hash)?;
  git_object.stdout(writer)?;
  Ok(())
}

pub fn write_tree(repo_path: &Path, stdout: &mut dyn io::Write) -> anyhow::Result<()> {
  let tree_object = GitObject::build_tree_object(repo_path)?;
  tree_object.write(repo_path)?;
  writeln!(stdout, "{}", tree_object.hash()?)?;
  Ok(())
}

pub fn ls_tree(
  object_hash: &str,
  writer: &mut dyn io::Write,
  repo_path: &Path,
  name_only: bool,
) -> anyhow::Result<()> {
  anyhow::ensure!(name_only, "only --name-only is supported");
  let git_object = GitObject::read_object(repo_path, object_hash)?;
  if git_object.kind != Kind::Tree {
    Err(anyhow::anyhow!("fatal: not a tree object"))?;
  }
  git_object.stdout(writer)?;
  Ok(())
}
