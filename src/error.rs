use std::error::Error as StdError;

#[derive(Debug)]
pub enum ErrorKind {
  ParserError,
}

impl std::fmt::Display for ErrorKind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ErrorKind::ParserError => write!(f, "ParserError"),
    }
  }
}

#[derive(Debug)]
pub struct Error {
  pub kind: ErrorKind,
  pub message: Option<String>,
  pub source: Option<Box<dyn StdError>>,
}

impl StdError for Error {
  fn source(&self) -> Option<&(dyn StdError + 'static)> {
    self.source.as_ref().and_then(|e| Some(e.as_ref()))
  }
}

impl std::fmt::Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match &self.source {
      None => {
        write!(
          f,
          "{}: {}",
          self.kind,
          self.message.clone().unwrap_or("None".to_string())
        )
      }
      Some(source) => {
        write!(
          f,
          "{}: {}\ncaused by {}",
          self.kind,
          self.message.clone().unwrap_or("None".to_string()),
          source
        )
      }
    }
  }
}

pub type Result<T> = std::result::Result<T, Error>;
