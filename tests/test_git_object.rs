use git_starter_rust::common::{GitObject, Kind};
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
  let git_object = GitObject::read_object(temp_dir, "557db03de997c86a4a028e1ebd3a1ceb225be238")?;
  assert_eq!(git_object.kind, git_starter_rust::common::Kind::Blob);
  assert_eq!(git_object.size, 12);
  assert_eq!(
    String::from_utf8(git_object.data.clone())?,
    "blob 12\0Hello World\n"
  );
  assert_eq!(
    git_object.hash()?,
    "557db03de997c86a4a028e1ebd3a1ceb225be238"
  );
  Ok(())
}

#[test]
pub fn test_tree_object_emtpy() -> anyhow::Result<()> {
  let temp_dir = TempDir::new("empty_dir")?;
  let temp_dir = temp_dir.path();
  let empty_tree = GitObject::build_tree_object(temp_dir)?;
  assert_eq!(empty_tree.kind, Kind::Tree);
  assert_eq!(empty_tree.size, 0);
  assert_eq!(empty_tree.data, "tree 0\x00".as_bytes());
  assert_eq!(
    empty_tree.hash()?,
    "4b825dc642cb6eb9a060e54bf8d69288fbee4904"
  );
  Ok(())
}

#[test]
pub fn test_git_with_single_file() -> anyhow::Result<()> {
  let temp_dir = TempDir::new("tmp_test_dir")?;
  let temp_dir = temp_dir.path();
  let root_file = temp_dir.join("root_file.txt");
  std::fs::write(root_file, b"root file content\n")?;
  let tree_object = GitObject::build_tree_object(temp_dir)?;
  assert_eq!(tree_object.kind, Kind::Tree);
  let mut expected_content = Vec::from("tree 41\x00100644 root_file.txt\x00".as_bytes());
  let file_sha: [u8; 20] = GitObject::_hash("blob 18\x00root file content\n".as_bytes())?;
  expected_content.extend_from_slice(&file_sha);
  assert_eq!(tree_object.data, expected_content);
  assert_eq!(
    tree_object.hash()?,
    "8bc36e1abc3de06227014c831f2a8d8e89bb7224"
  );
  Ok(())
}

#[test]
pub fn test_tree_object() -> anyhow::Result<()> {
  let temp_dir = TempDir::new("test_write_tree")?;
  let temp_dir = temp_dir.path();
  let subdir = temp_dir.join("subdir");
  std::fs::create_dir_all(&subdir)?;
  let subdir_file = subdir.join("subdir_file.txt");
  std::fs::write(subdir_file, b"subdir file content\n")?;
  let root_file = temp_dir.join("root_file.txt");
  std::fs::write(root_file, b"root file content\n")?;

  let tree_object = GitObject::build_tree_object(temp_dir)?;
  assert_eq!(tree_object.kind, Kind::Tree);
  let mut expected_data = Vec::from("tree 75\x00100644 root_file.txt\x00".as_bytes());
  expected_data.extend_from_slice(&GitObject::_hash(
    "blob 18\x00root file content\n".as_bytes(),
  )?);
  let subdir_tree = GitObject::build_tree_object(&subdir)?;
  expected_data.extend_from_slice("040000 subdir\x00".as_bytes());
  expected_data.extend_from_slice(&subdir_tree.hash_bytes()?);
  assert_eq!(tree_object.data, &expected_data[..]);
  Ok(())
}
