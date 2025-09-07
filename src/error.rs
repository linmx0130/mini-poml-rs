/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::error::Error as StdError;

#[derive(Debug)]
pub enum ErrorKind {
  ParserError,
  EvaluatorError,
  RendererError,
}

impl std::fmt::Display for ErrorKind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ErrorKind::ParserError => write!(f, "ParserError"),
      ErrorKind::EvaluatorError => write!(f, "EvaluatorError"),
      ErrorKind::RendererError => write!(f, "RendererError"),
    }
  }
}

#[derive(Debug)]
pub struct Error {
  pub kind: ErrorKind,
  pub message: String,
  pub source: Option<Box<dyn StdError>>,
}

impl StdError for Error {
  fn source(&self) -> Option<&(dyn StdError + 'static)> {
    self.source.as_ref().map(|e| e.as_ref())
  }
}

impl std::fmt::Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match &self.source {
      None => {
        write!(f, "{}: {}", self.kind, self.message)
      }
      Some(source) => {
        write!(f, "{}: {}\ncaused by {}", self.kind, self.message, source)
      }
    }
  }
}

pub type Result<T> = std::result::Result<T, Error>;
