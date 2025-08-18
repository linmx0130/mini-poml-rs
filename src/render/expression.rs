/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
use crate::error::{Error, ErrorKind, Result};

#[derive(Debug, PartialEq)]
pub enum ExpressionToken<'a> {
  // Reference - variables or keyword values.
  Ref(&'a [u8]),
  // Numbers
  Number(&'a [u8]),
  // Arithmetic operators
  ArithOp(&'a [u8]),
  // Left parenthesis
  LeftParenthesis,
  // Right parenthesis
  RightParenthesis,
  // Left bracket
  LeftBracket,
  // Right bracket
  RightBracket,
  // Left curly bracket
  LeftCurly,
  // Right curly bracket
  RightCurly,
}

pub fn tokenize_expression<'a>(buf: &'a [u8]) -> Result<Vec<ExpressionToken<'a>>> {
  let mut answer = Vec::new();
  let mut pos = 0;
  while pos < buf.len() {
    let c = u8_as_char(buf[pos])?;
    match c {
      c if c.is_alphabetic() || c == '_' => {
        // reference
        let mut ref_end_pos = pos + 1;
        while ref_end_pos < buf.len() {
          let nc = u8_as_char(buf[ref_end_pos])?;
          if nc.is_alphanumeric() || nc == '_' || nc == '.' {
            ref_end_pos += 1;
          } else {
            break;
          }
        }
        answer.push(ExpressionToken::Ref(&buf[pos..ref_end_pos]));
        pos = ref_end_pos;
      }
      c if c.is_numeric() => {
        // number
        let mut num_end_pos = pos + 1;
        while num_end_pos < buf.len() {
          let nc = u8_as_char(buf[num_end_pos])?;
          if nc.is_numeric() || nc == '.' {
            num_end_pos += 1;
          } else {
            break;
          }
        }
        answer.push(ExpressionToken::Number(&buf[pos..num_end_pos]));
        pos = num_end_pos;
      }
      '+' | '-' | '*' | '/' => {
        answer.push(ExpressionToken::ArithOp(&buf[pos..pos + 1]));
        pos += 1
      }
      '&' | '|' | '=' => {
        return Err(Error {
          kind: ErrorKind::EvaluatorError,
          message: format!("Operator has not been supported!"),
          source: None,
        });
      }
      '(' => {
        answer.push(ExpressionToken::LeftParenthesis);
        pos += 1;
      }
      ')' => {
        answer.push(ExpressionToken::RightParenthesis);
        pos += 1;
      }
      '[' => {
        answer.push(ExpressionToken::LeftBracket);
        pos += 1;
      }
      ']' => {
        answer.push(ExpressionToken::RightBracket);
        pos += 1;
      }
      '{' => {
        answer.push(ExpressionToken::LeftCurly);
        pos += 1;
      }
      '}' => {
        answer.push(ExpressionToken::RightCurly);
        pos += 1;
      }
      c if c.is_whitespace() => {
        pos += 1;
      }
      _ => {
        return Err(Error {
          kind: ErrorKind::EvaluatorError,
          message: format!("Invalid char encoutered in expression"),
          source: None,
        });
      }
    }
  }
  Ok(answer)
}

fn u8_as_char(v: u8) -> Result<char> {
  let Some(c) = char::from_u32(v.into()) else {
    return Err(Error {
      kind: ErrorKind::EvaluatorError,
      message: format!("Invalid char encoutered in expression"),
      source: None,
    });
  };
  Ok(c)
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn test_tokenize() {
    let expression = "(name.age + 1.5) * 2";
    let tokens = tokenize_expression(expression.as_bytes()).unwrap();
    assert_eq!(tokens[0], ExpressionToken::LeftParenthesis);
    assert_eq!(tokens[1], ExpressionToken::Ref("name.age".as_bytes()));
    assert_eq!(tokens[2], ExpressionToken::ArithOp("+".as_bytes()));
    assert_eq!(tokens[3], ExpressionToken::Number("1.5".as_bytes()));
    assert_eq!(tokens[4], ExpressionToken::RightParenthesis);
    assert_eq!(tokens[5], ExpressionToken::ArithOp("*".as_bytes()));
    assert_eq!(tokens[6], ExpressionToken::Number("2".as_bytes()));
  }
}
