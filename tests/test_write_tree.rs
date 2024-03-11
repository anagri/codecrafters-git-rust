use git_starter_rust::{command::write_tree, common::GitObject};
use tempdir::TempDir;

#[test]
pub fn test_write_tree() -> anyhow::Result<()> {
  let temp_dir = TempDir::new("test_write_tree")?;
  let temp_dir = temp_dir.path();
  let subdir = temp_dir.join("subdir");
  std::fs::create_dir_all(&subdir)?;
  let subdir_file = subdir.join("subdir_file.txt");
  std::fs::write(subdir_file, b"subdir file content\n")?;
  let root_file = temp_dir.join("root_file.txt");
  std::fs::write(root_file.clone(), b"root file content\n")?;
  let mut stdout = std::io::Cursor::new(Vec::<u8>::new());

  let tree_obj = GitObject::build_tree_object(temp_dir)?;
  let mut content = Vec::from("tree 75\x00100644 root_file.txt\x00".as_bytes());
  let root_file_obj = GitObject::build_file_object(&root_file)?;
  content.extend_from_slice(&root_file_obj.hash_bytes()?);
  content.extend_from_slice("040000 subdir\x00".as_bytes());
  let subdir_obj = GitObject::build_tree_object(&subdir)?;
  content.extend_from_slice(&subdir_obj.hash_bytes()?);
  assert_eq!(tree_obj.data, &content[..]);

  write_tree(temp_dir, &mut stdout)?;
  let expected_hash = format!("{}\n", tree_obj.hash()?);
  assert_eq!(
    String::from_utf8(stdout.into_inner()).unwrap(),
    expected_hash
  );
  Ok(())
}
