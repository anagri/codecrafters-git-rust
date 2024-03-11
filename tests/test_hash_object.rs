use git_starter_rust::command::hash_object;
use std::io::Cursor;
use tempdir::TempDir;

#[test]
pub fn test_hash_object() -> anyhow::Result<()> {
  let temp_dir = TempDir::new("tmp_test_dir")?;
  let temp_dir = temp_dir.path();
  let mut stdout = Cursor::new(Vec::<u8>::new());
  let readme_path = temp_dir.join("test.md");
  let readme_path = readme_path.as_path();
  std::fs::write(readme_path, b"Hello World\n")?;
  hash_object(readme_path, &mut stdout, false)?;
  assert_eq!(
    String::from_utf8(stdout.into_inner()).unwrap(),
    "557db03de997c86a4a028e1ebd3a1ceb225be238\n"
  );
  Ok(())
}

#[test]
pub fn test_hash_object_write() -> anyhow::Result<()> {
  let temp_dir = TempDir::new("tmp_test_dir")?;
  let temp_dir = temp_dir.path();
  let mut stdout = Cursor::new(Vec::<u8>::new());
  let readme_path = temp_dir.join("test.md");
  let readme_path = readme_path.as_path();
  std::fs::write(readme_path, b"Hello World\n")?;
  hash_object(readme_path, &mut stdout, true)?;
  assert_eq!(
    String::from_utf8(stdout.into_inner()).unwrap(),
    "557db03de997c86a4a028e1ebd3a1ceb225be238\n"
  );
  let git_obj = std::path::Path::new(".git/objects/55/7db03de997c86a4a028e1ebd3a1ceb225be238");
  assert!(git_obj.exists());
  Ok(())
}
