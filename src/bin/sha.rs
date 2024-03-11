use sha1::{Digest, Sha1};

pub fn main() -> anyhow::Result<()> {
  let mut hasher = Sha1::new();
  let content = std::fs::read("hash.md")?;
  hasher.update(content);
  let hash = hex::encode(hasher.finalize());
  println!("{}", hash);
  Ok(())
}
