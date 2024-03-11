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
  Ok(())
}
