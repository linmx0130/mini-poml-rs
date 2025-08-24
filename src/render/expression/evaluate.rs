/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use super::tokenize::ExpressionToken;
use super::utils::is_false_json_value;
use crate::error::{Error, ErrorKind, Result};
use crate::render::render_context::RenderContext;
use serde_json::Value;

pub fn evaluate_expression_tokens(
  tokens: &[ExpressionToken],
  context: &RenderContext,
) -> Result<Value> {
  let (value, next_pos) = evaluate_expression_value(tokens, 0, context)?;

  return if next_pos == tokens.len() {
    Ok(value)
  } else {
    Err(Error {
      kind: ErrorKind::EvaluatorError,
      message: format!("Not implemented"),
      source: None,
    })
  };
}

#[derive(Debug, Clone, PartialEq)]
enum ExpressionPart<'a> {
  Value(serde_json::Value),
  Operator(&'a str),
}

fn evaluate_expression_value(
  tokens: &[ExpressionToken],
  start_pos: usize,
  context: &RenderContext,
) -> Result<(Value, usize)> {
  let mut pos = start_pos;
  let mut parts: Vec<ExpressionPart> = vec![];
  while pos < tokens.len() {
    match tokens[pos] {
      // Signals of ending of a (sub) expression: ']', '}', ','
      ExpressionToken::RightBracket | ExpressionToken::RightCurly | ExpressionToken::Comma => break,
      // Arith operator
      ExpressionToken::ArithOp(op_name_buf) => {
        let op_name = str::from_utf8(op_name_buf).unwrap();
        pos = pos + 1;
        parts.push(ExpressionPart::Operator(op_name))
      }
      ExpressionToken::Exclamation => {
        if pos + 1 >= tokens.len() {
          return Err(Error {
            kind: ErrorKind::EvaluatorError,
            message: format!("No value after NOT operator"),
            source: None,
          });
        }
        let (value, next_pos) = recognize_next_value(tokens, pos + 1, context)?;
        parts.push(ExpressionPart::Value(Value::Bool(is_false_json_value(
          &value,
        ))));
        pos += next_pos;
      }
      ExpressionToken::Ref(_) | ExpressionToken::Number(_) | ExpressionToken::String(_) => {
        let (value, next_pos) = recognize_next_value(tokens, pos, context)?;
        parts.push(ExpressionPart::Value(value));
        pos = next_pos;
      }
      ExpressionToken::LeftBracket => {
        let (value, next_pos) = recognize_next_array(tokens, pos, context)?;
        parts.push(ExpressionPart::Value(value));
        pos = next_pos;
      }
      ExpressionToken::LeftCurly => {
        return Err(Error {
          kind: ErrorKind::EvaluatorError,
          message: format!("Not implemented to recognize objects"),
          source: None,
        });
      }
      _ => {
        return Err(Error {
          kind: ErrorKind::EvaluatorError,
          message: format!("Not implemented to recognize token: {:?}", tokens[pos]),
          source: None,
        });
      }
    }
  }
  parts = process_plus_and_minus_operators(parts)?;
  if parts.len() > 1 {
    return Err(Error {
      kind: ErrorKind::EvaluatorError,
      message: format!("Some operators remained unprocessed!"),
      source: None,
    });
  }
  let Some(ExpressionPart::Value(ret_value)) = parts.pop() else {
    return Err(Error {
      kind: ErrorKind::EvaluatorError,
      message: format!("No value found in the expression"),
      source: None,
    });
  };
  Ok((ret_value, pos))
}

fn process_plus_and_minus_operators<'a>(
  parts: Vec<ExpressionPart<'a>>,
) -> Result<Vec<ExpressionPart<'a>>> {
  let mut contain_plus_minus = false;
  for i in 0..parts.len() {
    if parts[i] == ExpressionPart::Operator("+") || parts[i] == ExpressionPart::Operator("-") {
      contain_plus_minus = true;
    }
  }

  // directly return if there is no +/- operators in the input
  if !contain_plus_minus {
    return Ok(parts);
  }

  let mut new_parts = Vec::new();
  let mut i = 0;
  while i < parts.len() {
    match parts[i] {
      ExpressionPart::Operator("+") => {
        let Some(ExpressionPart::Value(a)) = new_parts.pop() else {
          return Err(Error {
            kind: ErrorKind::EvaluatorError,
            message: format!("Operator + appears without a value before it."),
            source: None,
          });
        };
        let Some(ExpressionPart::Value(b)) = parts.get(i + 1) else {
          return Err(Error {
            kind: ErrorKind::EvaluatorError,
            message: format!("Operator + appears without a value after it."),
            source: None,
          });
        };
        let value = handle_plus_operator(&a, b)?;
        new_parts.push(ExpressionPart::Value(value));
        i += 2;
      }
      ExpressionPart::Operator("-") => {
        let Some(ExpressionPart::Value(a)) = new_parts.pop() else {
          return Err(Error {
            kind: ErrorKind::EvaluatorError,
            message: format!("Operator - appears without a value before it."),
            source: None,
          });
        };
        let Some(ExpressionPart::Value(b)) = parts.get(i + 1) else {
          return Err(Error {
            kind: ErrorKind::EvaluatorError,
            message: format!("Operator - appears without a value after it."),
            source: None,
          });
        };
        let value = handle_minus_operator(&a, b)?;
        new_parts.push(ExpressionPart::Value(value));
        i += 2;
      }
      _ => {
        new_parts.push(parts[i].clone());
        i = i + 1;
      }
    }
  }
  Ok(new_parts)
}

fn recognize_next_array(
  tokens: &[ExpressionToken],
  start_pos: usize,
  context: &RenderContext,
) -> Result<(Value, usize)> {
  let mut pos = start_pos + 1;
  let mut array_value: Vec<Value> = Vec::new();
  let mut array_finished = false;
  while pos < tokens.len() {
    if tokens[pos] == ExpressionToken::RightBracket {
      pos = pos + 1;
      array_finished = true;
      break;
    } else {
      // read next sub-expression
      let (item_value, next_pos) = evaluate_expression_value(tokens, pos, context)?;
      array_value.push(item_value);
      match tokens[next_pos] {
        ExpressionToken::Comma => {
          pos = next_pos + 1;
        }
        ExpressionToken::RightBracket => {
          pos = next_pos;
          continue;
        }
        _ => {
          return Err(Error {
            kind: ErrorKind::EvaluatorError,
            message: format!(
              "Expect comma ',' or right bracket ']' characters, but found {:?}",
              tokens[pos]
            ),
            source: None,
          });
        }
      };
    }
  }
  return if !array_finished {
    Err(Error {
      kind: ErrorKind::EvaluatorError,
      message: format!("Array value has not finished in the expression"),
      source: None,
    })
  } else {
    Ok((Value::Array(array_value), pos))
  };
}

fn recognize_next_value(
  tokens: &[ExpressionToken],
  pos: usize,
  context: &RenderContext,
) -> Result<(Value, usize)> {
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
      ExpressionToken::String(strc) => {
        let str_val = evaluate_string(strc)?;
        return Ok((str_val, pos + 1));
      }
      _ => {
        return Err(Error {
          kind: ErrorKind::EvaluatorError,
          message: format!("Expect a value token, but not found"),
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

fn evaluate_string(strc: &[u8]) -> Result<Value> {
  let str_val = match str::from_utf8(&strc[1..strc.len() - 1]) {
    Ok(s) => s,
    Err(e) => {
      return Err(Error {
        kind: ErrorKind::EvaluatorError,
        message: format!("Failed to decode string literal in expression."),
        source: Some(Box::new(e)),
      });
    }
  };
  Ok(Value::String(str_val.to_string()))
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

fn cast_as_i64(v: &Value) -> Option<i64> {
  match v {
    Value::Null => None,
    Value::Bool(b) => {
      if *b {
        Some(1)
      } else {
        Some(0)
      }
    }
    Value::Number(n) => n.as_i64(),
    Value::String(_) => None,
    Value::Array(_) => None,
    Value::Object(_) => None,
  }
}

fn cast_as_f64(v: &Value) -> Option<f64> {
  match v {
    Value::Null => None,
    Value::Bool(b) => {
      if *b {
        Some(1.0)
      } else {
        Some(0.0)
      }
    }
    Value::Number(n) => n.as_f64(),
    Value::String(_) => None,
    Value::Array(_) => None,
    Value::Object(_) => None,
  }
}

fn cast_as_string(v: &Value) -> Option<String> {
  match v {
    Value::Null => Some("null".to_string()),
    Value::Bool(b) => {
      if *b {
        Some("true".to_owned())
      } else {
        Some("false".to_owned())
      }
    }
    Value::Number(n) => Some(format!("{}", n)),
    Value::String(s) => Some(s.to_owned()),
    Value::Array(_) => None,
    Value::Object(_) => None,
  }
}

fn handle_plus_operator(a: &Value, b: &Value) -> Result<Value> {
  let int_a = cast_as_i64(a);
  let int_b = cast_as_i64(b);
  if int_a.is_some() && int_b.is_some() {
    return Ok(Value::Number(
      serde_json::Number::from_i128((int_a.unwrap() + int_b.unwrap()).into()).unwrap(),
    ));
  }
  let num_a = cast_as_f64(a);
  let num_b = cast_as_f64(b);
  if num_a.is_some() && num_b.is_some() {
    return Ok(Value::Number(
      serde_json::Number::from_f64(num_a.unwrap() + num_b.unwrap()).unwrap(),
    ));
  }
  let str_a = cast_as_string(&a);
  let str_b = cast_as_string(&b);
  if str_a.is_some() && str_b.is_some() {
    return Ok(Value::String(format!(
      "{}{}",
      str_a.unwrap(),
      str_b.unwrap()
    )));
  }
  return Err(Error {
    kind: ErrorKind::EvaluatorError,
    message: format!("Failed to perform plus operator on {:?} and {:?}.", a, b),
    source: None,
  });
}

fn handle_minus_operator(a: &Value, b: &Value) -> Result<Value> {
  let int_a = cast_as_i64(a);
  let int_b = cast_as_i64(b);
  if int_a.is_some() && int_b.is_some() {
    return Ok(Value::Number(
      serde_json::Number::from_i128((int_a.unwrap() - int_b.unwrap()).into()).unwrap(),
    ));
  }
  let num_a = cast_as_f64(a);
  let num_b = cast_as_f64(b);
  if num_a.is_some() && num_b.is_some() {
    return Ok(Value::Number(
      serde_json::Number::from_f64(num_a.unwrap() - num_b.unwrap()).unwrap(),
    ));
  }
  return Err(Error {
    kind: ErrorKind::EvaluatorError,
    message: format!("Failed to perform plus operator on {:?} and {:?}.", a, b),
    source: None,
  });
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

  #[test]
  fn test_evaluate_array() {
    let Value::Object(variables) = json!({}) else {
      panic!();
    };
    let context = RenderContext::from(variables);
    assert_eq!(
      evaluate_expression_tokens(
        &[
          ExpressionToken::LeftBracket,
          ExpressionToken::Number(b"1"),
          ExpressionToken::Comma,
          ExpressionToken::String(b"\'2\'"),
          ExpressionToken::Comma,
          ExpressionToken::LeftBracket,
          ExpressionToken::Ref(b"true"),
          ExpressionToken::Comma,
          ExpressionToken::Ref(b"false"),
          ExpressionToken::Comma,
          ExpressionToken::RightBracket,
          ExpressionToken::Comma,
          ExpressionToken::RightBracket,
        ],
        &context
      )
      .unwrap(),
      json!([1, "2", [true, false]])
    );
  }

  #[test]
  fn test_evaluate_not() {
    let Value::Object(variables) = json!({"flag": false}) else {
      panic!();
    };
    let context = RenderContext::from(variables);
    assert_eq!(
      evaluate_expression_tokens(
        &[ExpressionToken::Exclamation, ExpressionToken::Ref(b"flag"),],
        &context
      )
      .unwrap(),
      json!(true)
    );
  }

  #[test]
  fn test_evaluate_plus_arith() {
    let Value::Object(variables) = json!({
        "a": 1,
        "b": 2,
        "s": "a"
    }) else {
      panic!();
    };
    let context = RenderContext::from(variables);
    let (result, pos) = evaluate_expression_value(
      &[
        ExpressionToken::Ref(b"a"),
        ExpressionToken::ArithOp(b"+"),
        ExpressionToken::Ref(b"b"),
      ],
      0,
      &context,
    )
    .unwrap();
    assert_eq!(result, json!(3));
    assert_eq!(pos, 3);

    let (result, pos) = evaluate_expression_value(
      &[
        ExpressionToken::Ref(b"a"),
        ExpressionToken::ArithOp(b"+"),
        ExpressionToken::Ref(b"s"),
      ],
      0,
      &context,
    )
    .unwrap();
    assert_eq!(result, json!("1a"));
    assert_eq!(pos, 3);
  }

  #[test]
  fn test_evaluate_minus_arith() {
    let Value::Object(variables) = json!({
        "a": 1,
        "b": 2,
        "c": 3.0,
    }) else {
      panic!();
    };
    let context = RenderContext::from(variables);
    let (result, pos) = evaluate_expression_value(
      &[
        ExpressionToken::Ref(b"a"),
        ExpressionToken::ArithOp(b"-"),
        ExpressionToken::Ref(b"b"),
      ],
      0,
      &context,
    )
    .unwrap();
    assert_eq!(result, json!(-1));
    assert_eq!(pos, 3);

    let (result, pos) = evaluate_expression_value(
      &[
        ExpressionToken::Ref(b"a"),
        ExpressionToken::ArithOp(b"-"),
        ExpressionToken::Ref(b"c"),
      ],
      0,
      &context,
    )
    .unwrap();
    assert_eq!(result, json!(-2.0));
    assert_eq!(pos, 3);
  }
}
