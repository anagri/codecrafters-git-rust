use sha1::{Digest, Sha1};

pub fn main() -> anyhow::Result<()> {
  let mut hasher = Sha1::new();
  let content = std::fs::read("test.md")?;
  hasher.update(content);
  let hash = hasher.finalize();
  let hash = hash.as_slice();
  let hash: [u8; 20] = hash.try_into()?;
  println!("{:?}", hash);
  Ok(())
}
