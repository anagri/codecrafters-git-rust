use std::io::Cursor;

use anyhow::Ok;
use git_starter_rust::command::ls_tree;
use tempdir::TempDir;

#[test]
pub fn test_ls_tree() -> anyhow::Result<()> {
  let temp_dir = TempDir::new("tmp_test_dir")?;
  let temp_dir = temp_dir.path();
  let tree_object = include_bytes!("data/6ae106b480544288797befd3a2debb1f79f087");
  std::fs::create_dir_all(temp_dir.join(".git/objects/6a"))?;
  std::fs::write(
    temp_dir.join(".git/objects/6a/e106b480544288797befd3a2debb1f79f087"),
    tree_object,
  )?;
  let mut stdout = Cursor::new(Vec::<u8>::new());
  ls_tree(
    "6ae106b480544288797befd3a2debb1f79f087",
    &mut stdout,
    temp_dir,
    true,
  )?;
  let expected = r#".gitattributes
.gitignore
Cargo.lock
Cargo.toml
README.md
codecrafters.yml
src
your_git.sh
"#;
  assert_eq!(String::from_utf8(stdout.into_inner()).unwrap(), expected);
  Ok(())
}
