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
  // String literals
  String(&'a [u8]),
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
  // Comma
  Comma,
  // Colon
  Colon,
  // Exclamation mark !
  Exclamation,
  // Dot
  Dot,
  // Question mark ?
  QuestionMark,
}

pub fn tokenize_expression<'a>(buf: &'a [u8]) -> Result<Vec<ExpressionToken<'a>>> {
  let mut answer = Vec::new();
  let mut pos = 0;
  while pos < buf.len() {
    let c = u8_as_char(buf[pos])?;
    match c {
      c if c.is_alphabetic() || c == '_' => {
        let ref_end_pos = seek_ref_end(buf, pos)?;
        let ref_name = &buf[pos..ref_end_pos];
        if ref_name == b"in" {
          answer.push(ExpressionToken::ArithOp(&buf[pos..ref_end_pos]));
        } else {
          answer.push(ExpressionToken::Ref(&buf[pos..ref_end_pos]));
        }
        pos = ref_end_pos;
      }
      c if c.is_numeric() => {
        let num_end_pos = seek_number_end(buf, pos)?;
        answer.push(ExpressionToken::Number(&buf[pos..num_end_pos]));
        pos = num_end_pos;
      }
      '.' => {
        if pos + 1 >= buf.len() {
          return Err(Error {
            kind: ErrorKind::EvaluatorError,
            message: "No content following dot operator.".to_string(),
            source: None,
          });
        }
        let nc = u8_as_char(buf[pos + 1])?;
        if nc.is_numeric() {
          let num_end_pos = seek_number_end(buf, pos)?;
          answer.push(ExpressionToken::Number(&buf[pos..num_end_pos]));
          pos = num_end_pos;
        } else {
          answer.push(ExpressionToken::Dot);
          pos += 1;
        }
      }
      '"' | '\'' => {
        let string_end_pos = seek_string_end(buf, pos)?;
        answer.push(ExpressionToken::String(&buf[pos..string_end_pos]));
        pos = string_end_pos;
      }
      '+' | '-' | '*' | '/' | '%' => {
        answer.push(ExpressionToken::ArithOp(&buf[pos..pos + 1]));
        pos += 1
      }
      '&' | '|' => {
        if pos + 1 < buf.len() && buf[pos + 1] == buf[pos] {
          answer.push(ExpressionToken::ArithOp(&buf[pos..pos + 2]));
          pos += 2;
        } else {
          return Err(Error {
            kind: ErrorKind::EvaluatorError,
            message: "Operator has not been supported!".to_string(),
            source: None,
          });
        }
      }
      '>' | '<' => {
        if pos + 1 < buf.len() && buf[pos + 1] == b'=' {
          answer.push(ExpressionToken::ArithOp(&buf[pos..pos + 2]));
          pos += 2;
        } else {
          answer.push(ExpressionToken::ArithOp(&buf[pos..pos + 1]));
          pos += 1;
        }
      }
      '=' => {
        if pos + 2 < buf.len() && buf[pos + 1] == b'=' && buf[pos + 2] == b'=' {
          answer.push(ExpressionToken::ArithOp(&buf[pos..pos + 3]));
          pos += 3;
        } else {
          return Err(Error {
            kind: ErrorKind::EvaluatorError,
            message: "Operator has not been supported!".to_string(),
            source: None,
          });
        }
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
      ',' => {
        answer.push(ExpressionToken::Comma);
        pos += 1;
      }
      ':' => {
        answer.push(ExpressionToken::Colon);
        pos += 1;
      }
      '?' => {
        answer.push(ExpressionToken::QuestionMark);
        pos += 1;
      }
      '!' => {
        if pos + 2 < buf.len() && buf[pos + 1] == b'=' && buf[pos + 2] == b'=' {
          answer.push(ExpressionToken::ArithOp(&buf[pos..pos + 3]));
          pos += 3;
        } else {
          answer.push(ExpressionToken::Exclamation);
          pos += 1;
        }
      }
      c if c.is_whitespace() => {
        pos += 1;
      }
      _ => {
        return Err(Error {
          kind: ErrorKind::EvaluatorError,
          message: "Invalid char encoutered in expression".to_string(),
          source: None,
        });
      }
    }
  }
  Ok(answer)
}

/**
 * Seek the end of the current reference token. Must be called with `buf[pos]` as the start of
 * the reference token.
 *
 * Return the end position.
 */
fn seek_ref_end(buf: &[u8], pos: usize) -> Result<usize> {
  // reference
  let mut ref_end_pos = pos + 1;
  while ref_end_pos < buf.len() {
    let nc = u8_as_char(buf[ref_end_pos])?;
    if nc.is_alphanumeric() || nc == '_' {
      ref_end_pos += 1;
    } else {
      break;
    }
  }
  Ok(ref_end_pos)
}

fn seek_number_end(buf: &[u8], pos: usize) -> Result<usize> {
  let mut found_dot = false;
  // number
  let mut num_end_pos = pos;
  while num_end_pos < buf.len() {
    let nc = u8_as_char(buf[num_end_pos])?;
    if nc.is_numeric() {
      num_end_pos += 1;
    } else if nc == '.' {
      if !found_dot {
        found_dot = true;
        num_end_pos += 1;
      } else {
        return Err(Error {
          kind: ErrorKind::EvaluatorError,
          message: "Multiple dots found in a number literal.".to_string(),
          source: None,
        });
      }
    } else {
      break;
    }
  }
  Ok(num_end_pos)
}

fn seek_string_end(buf: &[u8], pos: usize) -> Result<usize> {
  let quote = buf[pos];
  let mut cur = pos + 1;

  while cur < buf.len() {
    if buf[cur] == quote {
      return Ok(cur + 1);
    }
    if buf[cur] == b'\\' {
      cur += 2;
    } else {
      cur += 1;
    }
  }

  Err(Error {
    kind: ErrorKind::EvaluatorError,
    message: "String literal doesn't end in the expression.".to_string(),
    source: None,
  })
}

fn u8_as_char(v: u8) -> Result<char> {
  let Some(c) = char::from_u32(v.into()) else {
    return Err(Error {
      kind: ErrorKind::EvaluatorError,
      message: "Invalid char encoutered in expression".to_string(),
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
    let expression = "(name.age + 1.5) * .2";
    let tokens = tokenize_expression(expression.as_bytes()).unwrap();
    assert_eq!(tokens[0], ExpressionToken::LeftParenthesis);
    assert_eq!(tokens[1], ExpressionToken::Ref("name".as_bytes()));
    assert_eq!(tokens[2], ExpressionToken::Dot);
    assert_eq!(tokens[3], ExpressionToken::Ref("age".as_bytes()));
    assert_eq!(tokens[4], ExpressionToken::ArithOp("+".as_bytes()));
    assert_eq!(tokens[5], ExpressionToken::Number("1.5".as_bytes()));
    assert_eq!(tokens[6], ExpressionToken::RightParenthesis);
    assert_eq!(tokens[7], ExpressionToken::ArithOp("*".as_bytes()));
    assert_eq!(tokens[8], ExpressionToken::Number(".2".as_bytes()));
  }

  #[test]
  fn test_tokenize_array() {
    let expression = "['apple', 'banana\\\'', \"orange\", ]";
    let tokens = tokenize_expression(expression.as_bytes()).unwrap();
    assert_eq!(tokens[0], ExpressionToken::LeftBracket);
    assert_eq!(tokens[1], ExpressionToken::String(b"'apple'"));
    assert_eq!(tokens[2], ExpressionToken::Comma);
    assert_eq!(tokens[3], ExpressionToken::String(b"'banana\\''"));
    assert_eq!(tokens[4], ExpressionToken::Comma);
    assert_eq!(tokens[5], ExpressionToken::String(b"\"orange\""));
    assert_eq!(tokens[6], ExpressionToken::Comma);
    assert_eq!(tokens[7], ExpressionToken::RightBracket);
  }

  #[test]
  fn test_tokenize_logic_expression() {
    let expression = "(name.age + 1.5) * (true && name.count === 1)";
    let tokens = tokenize_expression(expression.as_bytes()).unwrap();
    assert_eq!(
      tokens,
      [
        ExpressionToken::LeftParenthesis,
        ExpressionToken::Ref(b"name"),
        ExpressionToken::Dot,
        ExpressionToken::Ref(b"age"),
        ExpressionToken::ArithOp(b"+"),
        ExpressionToken::Number(b"1.5"),
        ExpressionToken::RightParenthesis,
        ExpressionToken::ArithOp(b"*"),
        ExpressionToken::LeftParenthesis,
        ExpressionToken::Ref(b"true"),
        ExpressionToken::ArithOp(b"&&"),
        ExpressionToken::Ref(b"name"),
        ExpressionToken::Dot,
        ExpressionToken::Ref(b"count"),
        ExpressionToken::ArithOp(b"==="),
        ExpressionToken::Number(b"1"),
        ExpressionToken::RightParenthesis
      ]
    );
  }

  #[test]
  fn test_tokenize_not_equal() {
    let expression = "a !== b";
    let tokens = tokenize_expression(expression.as_bytes()).unwrap();
    assert_eq!(
      tokens,
      [
        ExpressionToken::Ref(b"a"),
        ExpressionToken::ArithOp(b"!=="),
        ExpressionToken::Ref(b"b"),
      ]
    );
  }

  #[test]
  fn test_tokenize_in_operator() {
    let expression = "a in b";
    let tokens = tokenize_expression(expression.as_bytes()).unwrap();
    assert_eq!(
      tokens,
      [
        ExpressionToken::Ref(b"a"),
        ExpressionToken::ArithOp(b"in"),
        ExpressionToken::Ref(b"b"),
      ]
    );
  }
}
