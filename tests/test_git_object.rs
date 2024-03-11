use git_starter_rust::common::GitObject;
use tempdir::TempDir;

#[test]
pub fn test_git_object() -> anyhow::Result<()> {
  let temp_dir = TempDir::new("tmp_test_dir")?;
  let temp_dir = temp_dir.path();
  let git_object = include_bytes!("data/557db03de997c86a4a028e1ebd3a1ceb225be238");
  std::fs::create_dir_all(temp_dir.join(".git/objects/55"))?;
  std::fs::write(
    temp_dir.join(".git/objects/55/7db03de997c86a4a028e1ebd3a1ceb225be238"),
    git_object,
  )?;
  let git_object = GitObject::read_object("557db03de997c86a4a028e1ebd3a1ceb225be238")?;
  assert_eq!(git_object.kind, git_starter_rust::common::Kind::Blob);
  assert_eq!(git_object.size, 12);
  assert_eq!(String::from_utf8(git_object.data.clone())?, "blob 12\0Hello World\n");
  assert_eq!(
    git_object.hash()?,
    "557db03de997c86a4a028e1ebd3a1ceb225be238"
  );
  Ok(())
}
