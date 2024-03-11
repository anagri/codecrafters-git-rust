use std::io::Cursor;

use git_starter_rust::command::cat_file;
use tempdir::TempDir;

#[test]
pub fn test_cat_file() -> anyhow::Result<()> {
  let temp_dir = TempDir::new("tmp_test_dir")?;
  let temp_dir = temp_dir.path();
  let mut stdout = Cursor::new(Vec::<u8>::new());
  let git_object = include_bytes!("data/557db03de997c86a4a028e1ebd3a1ceb225be238");
  std::fs::create_dir_all(temp_dir.join(".git/objects/55"))?;
  std::fs::write(
    temp_dir.join(".git/objects/55/7db03de997c86a4a028e1ebd3a1ceb225be238"),
    git_object,
  )?;
  cat_file(
    "557db03de997c86a4a028e1ebd3a1ceb225be238",
    &mut stdout,
    temp_dir,
    true,
  )?;
  assert_eq!(
    String::from_utf8(stdout.into_inner()).unwrap(),
    "Hello World\n"
  );
  Ok(())
}
