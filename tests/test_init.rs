use git_starter_rust::command::init;
use std::{env, io::Cursor};
use tempdir::TempDir;
use walkdir::WalkDir;

#[test]
pub fn test_init() -> anyhow::Result<()> {
  let temp_dir = TempDir::new("tmp_test_dir")?;
  env::set_current_dir(&temp_dir)?;
  let mut writer = Cursor::new(Vec::new());
  init(temp_dir.path(), &mut writer)?;
  assert_eq!(
    String::from_utf8(writer.into_inner()).unwrap(),
    "Initialized git directory"
  );
  let mut temp_dir_files = WalkDir::new(temp_dir.path())
    .into_iter()
    .map(|entry| entry.unwrap())
    .map(|entry| entry.path().to_string_lossy().to_string())
    .collect::<Vec<_>>();
  temp_dir_files.sort();
  assert_eq!(5, temp_dir_files.len());
  let mut expected_dirs = ["", ".git", ".git/objects", ".git/refs", ".git/HEAD"]
    .map(|entry| {
      if entry.is_empty() {
        temp_dir.path().to_string_lossy().to_string()
      } else {
        temp_dir.path().join(entry).to_string_lossy().to_string()
      }
    })
    .to_vec();
  expected_dirs.sort();
  assert_eq!(expected_dirs, temp_dir_files);
  Ok(())
}
