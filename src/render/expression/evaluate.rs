/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use super::tokenize::ExpressionToken;
use crate::error::{Error, ErrorKind, Result};
use crate::render::render_context::RenderContext;
use serde_json::Value;

pub fn evaluate_expression_tokens(
  tokens: &[ExpressionToken],
  context: &RenderContext,
) -> Result<Value> {
  let mut pos = 0;
  while pos < tokens.len() {
    let (value, next_pos) = recognize_next_value(tokens, pos, context)?;
    pos = next_pos;
    // TODO: process expression that is not only a single value
    return Ok(value);
  }
  return Err(Error {
    kind: ErrorKind::EvaluatorError,
    message: format!("Not implemented"),
    source: None,
  });
}

fn recognize_next_value(
  tokens: &[ExpressionToken],
  start_pos: usize,
  context: &RenderContext,
) -> Result<(Value, usize)> {
  let mut pos = start_pos;
  while pos < tokens.len() {
    let cur = &tokens[pos];
    match cur {
      ExpressionToken::Ref(refc) => {
        let value = evaluate_reference(refc, context)?;
        return Ok((value, pos + 1));
      }
      ExpressionToken::Number(numc) => {
        let value = evaluate_number(numc)?;
        return Ok((value, pos + 1));
      }
      _ => {
        return Err(Error {
          kind: ErrorKind::EvaluatorError,
          message: format!("Not implemented"),
          source: None,
        });
      }
    }
  }
  return Err(Error {
    kind: ErrorKind::EvaluatorError,
    message: format!("Not implemented"),
    source: None,
  });
}

fn evaluate_reference(refc: &[u8], context: &RenderContext) -> Result<Value> {
  if match_u8_str(refc, "true") {
    return Ok(Value::Bool(true));
  }
  if match_u8_str(refc, "false") {
    return Ok(Value::Bool(false));
  }
  if match_u8_str(refc, "null") {
    return Ok(Value::Null);
  }
  let refs = match str::from_utf8(refc) {
    Ok(s) => s,
    Err(e) => {
      return Err(Error {
        kind: ErrorKind::EvaluatorError,
        message: format!("String decode error"),
        source: Some(Box::new(e)),
      });
    }
  };
  let mut parts = refs.split('.');
  let first_part = parts.next().unwrap();
  let null_value = Value::Null;
  let mut value_ref = match context.get_value(first_part) {
    Some(ref r) => r,
    None => &null_value,
  };
  let mut recognized_name = first_part.to_owned();

  while let Some(key_name) = parts.next() {
    match value_ref {
      Value::Null => {
        return Err(Error {
          kind: ErrorKind::EvaluatorError,
          message: format!(
            "Tried to access field `{}` on undefined or null variable `{}`.",
            key_name, recognized_name
          ),
          source: None,
        });
      }

      Value::Object(obj) => match obj.get(key_name) {
        Some(field_ref) => {
          value_ref = field_ref;
        }
        None => {
          value_ref = &null_value;
        }
      },

      _ => {
        return Err(Error {
          kind: ErrorKind::EvaluatorError,
          message: format!(
            "Variable `{}` is not an object and `{}` is not available on it",
            recognized_name, key_name
          ),
          source: None,
        });
      }
    }
    recognized_name += ".";
    recognized_name += key_name;
  }

  return Ok(value_ref.clone());
}

fn evaluate_number(numc: &[u8]) -> Result<Value> {
  let nums = str::from_utf8(numc).unwrap();

  if !numc.contains(&b'.') {
    let Ok(val): std::result::Result<i64, _> = str::parse(nums) else {
      return Err(Error {
        kind: ErrorKind::EvaluatorError,
        message: format!("Failed to parse number: {}", nums),
        source: None,
      });
    };
    Ok(Value::Number(
      serde_json::Number::from_i128(val.into()).unwrap(),
    ))
  } else {
    let Ok(val): std::result::Result<f64, _> = str::parse(nums) else {
      return Err(Error {
        kind: ErrorKind::EvaluatorError,
        message: format!("Failed to parse number: {}", nums),
        source: None,
      });
    };
    Ok(Value::Number(
      serde_json::Number::from_f64(val.into()).unwrap(),
    ))
  }
}

fn match_u8_str(src: &[u8], pat: &str) -> bool {
  let p = pat.as_bytes();
  if src.len() != p.len() {
    return false;
  }
  for i in 0..src.len() {
    if src[i] != p[i] {
      return false;
    }
  }
  true
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde_json::json;

  #[test]
  fn test_evaluate_reference() {
    let Value::Object(variables) = json!({
        "my": {"home":"127.0.0.1"},
        "count": 5
    }) else {
      panic!();
    };
    let context = RenderContext::from(variables);
    assert_eq!(
      evaluate_reference("my.home".as_bytes(), &context).unwrap(),
      json!("127.0.0.1")
    );
    assert_eq!(
      evaluate_reference("count".as_bytes(), &context).unwrap(),
      json!(5)
    );
    assert_eq!(
      evaluate_reference("my.car".as_bytes(), &context).unwrap(),
      Value::Null
    );
    assert!(evaluate_reference("my.car.window".as_bytes(), &context).is_err());
  }

  #[test]
  fn test_evaluate_numbers() {
    assert_eq!(evaluate_number("123".as_bytes()).unwrap(), json!(123));
    assert_eq!(evaluate_number("0.55".as_bytes()).unwrap(), json!(0.55));
  }
}
