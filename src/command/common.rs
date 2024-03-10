use std::fmt::Display;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Kind {
  Blob,
  Tree,
}

impl Display for Kind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Kind::Blob => write!(f, "blob"),
      Kind::Tree => write!(f, "tree"),
    }
  }
}
