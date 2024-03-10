use anyhow::Context;

use crate::command::common::{GitObject, Kind};

pub(crate) fn cat_file(pretty_print: bool, object_hash: String) -> anyhow::Result<()> {
  anyhow::ensure!(pretty_print, "only supports pretty print");
  let mut git_object = GitObject::read_object(object_hash)?;
  match &git_object.kind {
    Kind::Blob => {
      let stdout = std::io::stdout();
      let mut stdout = stdout.lock();
      let n = std::io::copy(&mut git_object.reader, &mut stdout)
        .context("copying from .git/objects to stdout")?;
      anyhow::ensure!(
        n == git_object.size,
        ".git/object file not of expected size, expected: '{}', actual: '{}'",
        git_object.size,
        n
      );
    }
    Kind::Tree => anyhow::bail!("only supports cat-file for blob only"),
  }
  Ok(())
}
