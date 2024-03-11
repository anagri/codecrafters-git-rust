use std::io::Cursor;

use git_starter_rust::{
  command::{commit_tree, init},
  common::GitObject,
};
use tempdir::TempDir;

#[test]
pub fn test_commit() -> anyhow::Result<()> {
  let temp_dir = TempDir::new("test_commit")?;
  let temp_dir = temp_dir.path();
  let mut stdout = Cursor::new(Vec::new());
  init(temp_dir, &mut stdout)?;
  let root_file = temp_dir.join("root_file.txt");
  std::fs::write(root_file.clone(), b"root file content\n")?;
  let tree_object = GitObject::build_tree_object(temp_dir)?;
  tree_object.write(temp_dir)?;

  let mut stdout = Cursor::new(Vec::new());
  commit_tree(
    tree_object.hash()?,
    &mut stdout,
    temp_dir,
    "test message",
    None,
  )?;

  let actual_output = String::from_utf8(stdout.into_inner()).unwrap();
  assert_eq!(actual_output, "05e1fd875f25ab9683c3b40e51016abc33ee6720\n");
  Ok(())
}
