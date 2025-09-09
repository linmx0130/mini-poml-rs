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

  if next_pos == tokens.len() {
    Ok(value)
  } else {
    Err(Error {
      kind: ErrorKind::EvaluatorError,
      message: "Not implemented".to_string(),
      source: None,
    })
  }
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
      ExpressionToken::RightBracket
      | ExpressionToken::RightCurly
      | ExpressionToken::Comma
      | ExpressionToken::RightParenthesis => break,
      ExpressionToken::LeftParenthesis => {
        let (value, new_pos) = evaluate_expression_value(tokens, pos + 1, context)?;
        if new_pos >= tokens.len() || tokens[new_pos] != ExpressionToken::RightParenthesis {
          return Err(Error {
            kind: ErrorKind::EvaluatorError,
            message: "Not paired right parenthesis for a left parenthesis".to_string(),
            source: None,
          });
        }
        pos = new_pos + 1;
        parts.push(ExpressionPart::Value(value));
      }
      // Arith operator
      ExpressionToken::ArithOp(op_name_buf) => {
        let op_name = str::from_utf8(op_name_buf).unwrap();
        pos += 1;
        parts.push(ExpressionPart::Operator(op_name))
      }
      ExpressionToken::Exclamation => {
        pos += 1;
        parts.push(ExpressionPart::Operator("!"));
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
          message: "Not implemented to recognize objects".to_string(),
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
  parts = process_not_operators(parts)?;
  parts = process_times_and_divide_operators(parts)?;
  parts = process_plus_and_minus_operators(parts)?;
  parts = process_equality_operators(parts)?;
  parts = process_and_operators(parts)?;
  parts = process_or_operators(parts)?;
  if parts.len() > 1 {
    return Err(Error {
      kind: ErrorKind::EvaluatorError,
      message: "Some operators remained unprocessed!".to_string(),
      source: None,
    });
  }
  let Some(ExpressionPart::Value(ret_value)) = parts.pop() else {
    return Err(Error {
      kind: ErrorKind::EvaluatorError,
      message: "No value found in the expression".to_string(),
      source: None,
    });
  };
  Ok((ret_value, pos))
}

fn process_and_operators<'a>(parts: Vec<ExpressionPart<'a>>) -> Result<Vec<ExpressionPart<'a>>> {
  let mut contain_and = false;
  for part in &parts {
    if *part == ExpressionPart::Operator("&&") {
      contain_and = true;
      break;
    }
  }

  // directly return if there is no and operators in the input
  if !contain_and {
    return Ok(parts);
  }

  let mut new_parts = Vec::new();
  let mut i = 0;
  while i < parts.len() {
    match parts[i] {
      ExpressionPart::Operator("&&") => {
        let Some(ExpressionPart::Value(a)) = new_parts.pop() else {
          return Err(Error {
            kind: ErrorKind::EvaluatorError,
            message: "Operator && appears without a value before it.".to_string(),
            source: None,
          });
        };
        let Some(ExpressionPart::Value(b)) = parts.get(i + 1) else {
          return Err(Error {
            kind: ErrorKind::EvaluatorError,
            message: "Operator && appears without a value after it.".to_string(),
            source: None,
          });
        };
        let mut value = true;
        if is_false_json_value(&a) {
          value = false;
        }
        if is_false_json_value(b) {
          value = false;
        }
        new_parts.push(ExpressionPart::Value(Value::Bool(value)));
        i += 2;
      }
      _ => {
        new_parts.push(parts[i].clone());
        i += 1;
      }
    }
  }
  Ok(new_parts)
}

fn process_or_operators<'a>(parts: Vec<ExpressionPart<'a>>) -> Result<Vec<ExpressionPart<'a>>> {
  let mut contain_or = false;
  for part in &parts {
    if *part == ExpressionPart::Operator("||") {
      contain_or = true;
      break;
    }
  }

  // directly return if there is no or operators in the input
  if !contain_or {
    return Ok(parts);
  }

  let mut new_parts = Vec::new();
  let mut i = 0;
  while i < parts.len() {
    match parts[i] {
      ExpressionPart::Operator("||") => {
        let Some(ExpressionPart::Value(a)) = new_parts.pop() else {
          return Err(Error {
            kind: ErrorKind::EvaluatorError,
            message: "Operator || appears without a value before it.".to_string(),
            source: None,
          });
        };
        let Some(ExpressionPart::Value(b)) = parts.get(i + 1) else {
          return Err(Error {
            kind: ErrorKind::EvaluatorError,
            message: "Operator || appears without a value after it.".to_string(),
            source: None,
          });
        };
        let mut value = false;
        if !is_false_json_value(&a) {
          value = true;
        }
        if !is_false_json_value(b) {
          value = true;
        }
        new_parts.push(ExpressionPart::Value(Value::Bool(value)));
        i += 2;
      }
      _ => {
        new_parts.push(parts[i].clone());
        i += 1;
      }
    }
  }
  Ok(new_parts)
}

fn process_not_operators<'a>(parts: Vec<ExpressionPart<'a>>) -> Result<Vec<ExpressionPart<'a>>> {
  let mut contain_not = false;
  for part in &parts {
    if *part == ExpressionPart::Operator("!") {
      contain_not = true;
    }
  }

  // directly return if there is no not operators in the input
  if !contain_not {
    return Ok(parts);
  }

  let mut new_parts = Vec::new();
  let mut i = 0;
  while i < parts.len() {
    match parts[i] {
      ExpressionPart::Operator("!") => {
        let Some(ExpressionPart::Value(b)) = parts.get(i + 1) else {
          return Err(Error {
            kind: ErrorKind::EvaluatorError,
            message: "Operator ! appears without a value after it.".to_string(),
            source: None,
          });
        };
        let value = is_false_json_value(b);
        new_parts.push(ExpressionPart::Value(Value::Bool(value)));
        i += 2;
      }
      _ => {
        new_parts.push(parts[i].clone());
        i += 1;
      }
    }
  }
  Ok(new_parts)
}
fn process_plus_and_minus_operators<'a>(
  parts: Vec<ExpressionPart<'a>>,
) -> Result<Vec<ExpressionPart<'a>>> {
  let mut contain_plus_minus = false;
  for part in &parts {
    if *part == ExpressionPart::Operator("+") || *part == ExpressionPart::Operator("-") {
      contain_plus_minus = true;
      break;
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
            message: "Operator + appears without a value before it.".to_string(),
            source: None,
          });
        };
        let Some(ExpressionPart::Value(b)) = parts.get(i + 1) else {
          return Err(Error {
            kind: ErrorKind::EvaluatorError,
            message: "Operator + appears without a value after it.".to_string(),
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
            message: "Operator - appears without a value before it.".to_string(),
            source: None,
          });
        };
        let Some(ExpressionPart::Value(b)) = parts.get(i + 1) else {
          return Err(Error {
            kind: ErrorKind::EvaluatorError,
            message: "Operator - appears without a value after it.".to_string(),
            source: None,
          });
        };
        let value = handle_minus_operator(&a, b)?;
        new_parts.push(ExpressionPart::Value(value));
        i += 2;
      }
      _ => {
        new_parts.push(parts[i].clone());
        i += 1;
      }
    }
  }
  Ok(new_parts)
}

fn process_times_and_divide_operators<'a>(
  parts: Vec<ExpressionPart<'a>>,
) -> Result<Vec<ExpressionPart<'a>>> {
  let mut contain_times_divide = false;
  for part in &parts {
    if *part == ExpressionPart::Operator("*") || *part == ExpressionPart::Operator("/") {
      contain_times_divide = true;
      break;
    }
  }

  // directly return if there is no * or / operators in the input
  if !contain_times_divide {
    return Ok(parts);
  }

  let mut new_parts = Vec::new();
  let mut i = 0;
  while i < parts.len() {
    match parts[i] {
      ExpressionPart::Operator("*") => {
        let Some(ExpressionPart::Value(a)) = new_parts.pop() else {
          return Err(Error {
            kind: ErrorKind::EvaluatorError,
            message: "Operator * appears without a value before it.".to_string(),
            source: None,
          });
        };
        let Some(ExpressionPart::Value(b)) = parts.get(i + 1) else {
          return Err(Error {
            kind: ErrorKind::EvaluatorError,
            message: "Operator * appears without a value after it.".to_string(),
            source: None,
          });
        };
        let value = handle_times_operator(&a, b)?;
        new_parts.push(ExpressionPart::Value(value));
        i += 2;
      }
      ExpressionPart::Operator("/") => {
        let Some(ExpressionPart::Value(a)) = new_parts.pop() else {
          return Err(Error {
            kind: ErrorKind::EvaluatorError,
            message: "Operator / appears without a value before it.".to_string(),
            source: None,
          });
        };
        let Some(ExpressionPart::Value(b)) = parts.get(i + 1) else {
          return Err(Error {
            kind: ErrorKind::EvaluatorError,
            message: "Operator - appears without a value after it.".to_string(),
            source: None,
          });
        };
        let value = handle_divide_operator(&a, b)?;
        new_parts.push(ExpressionPart::Value(value));
        i += 2;
      }
      _ => {
        new_parts.push(parts[i].clone());
        i += 1;
      }
    }
  }
  Ok(new_parts)
}

fn process_equality_operators<'a>(
  parts: Vec<ExpressionPart<'a>>,
) -> Result<Vec<ExpressionPart<'a>>> {
  let mut contain_equals = false;
  for part in &parts {
    if *part == ExpressionPart::Operator("===") || *part == ExpressionPart::Operator("!==") {
      contain_equals = true;
      break;
    }
  }

  // directly return if there is no equality operators in the input
  if !contain_equals {
    return Ok(parts);
  }

  let mut new_parts = Vec::new();
  let mut i = 0;
  while i < parts.len() {
    match parts[i] {
      ExpressionPart::Operator("===") => {
        let Some(ExpressionPart::Value(a)) = new_parts.pop() else {
          return Err(Error {
            kind: ErrorKind::EvaluatorError,
            message: "Operator === appears without a value before it.".to_string(),
            source: None,
          });
        };
        let Some(ExpressionPart::Value(b)) = parts.get(i + 1) else {
          return Err(Error {
            kind: ErrorKind::EvaluatorError,
            message: "Operator === appears without a value after it.".to_string(),
            source: None,
          });
        };
        let value = a == *b;
        new_parts.push(ExpressionPart::Value(Value::Bool(value)));
        i += 2;
      }
      ExpressionPart::Operator("!==") => {
        let Some(ExpressionPart::Value(a)) = new_parts.pop() else {
          return Err(Error {
            kind: ErrorKind::EvaluatorError,
            message: "Operator !== appears without a value before it.".to_string(),
            source: None,
          });
        };
        let Some(ExpressionPart::Value(b)) = parts.get(i + 1) else {
          return Err(Error {
            kind: ErrorKind::EvaluatorError,
            message: "Operator !== appears without a value after it.".to_string(),
            source: None,
          });
        };
        let value = a != *b;
        new_parts.push(ExpressionPart::Value(Value::Bool(value)));
        i += 2;
      }
      _ => {
        new_parts.push(parts[i].clone());
        i += 1;
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
      pos += 1;
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
  if !array_finished {
    Err(Error {
      kind: ErrorKind::EvaluatorError,
      message: "Array value has not finished in the expression".to_string(),
      source: None,
    })
  } else {
    Ok((Value::Array(array_value), pos))
  }
}

fn recognize_next_value(
  tokens: &[ExpressionToken],
  pos: usize,
  context: &RenderContext,
) -> Result<(Value, usize)> {
  let mut pos = pos;
  if pos < tokens.len() {
    let cur = &tokens[pos];
    match cur {
      ExpressionToken::Ref(refc) => {
        let value = evaluate_reference(refc, context)?;
        let mut value_ref = &value;
        let null_value = Value::Null;
        let mut recognized_name = String::from_utf8(refc.to_vec()).unwrap();
        pos += 1;
        while pos < tokens.len() {
          match tokens[pos] {
            ExpressionToken::Dot => {
              let Some(ExpressionToken::Ref(key_bytes)) = tokens.get(pos + 1) else {
                return Err(Error {
                  kind: ErrorKind::EvaluatorError,
                  message: "No reference found after dot.".to_string(),
                  source: None,
                });
              };

              let key_name = str::from_utf8(key_bytes).unwrap();
              recognized_name = recognized_name + "." + key_name;

              match value_ref {
                Value::Null => {
                  return Err(Error {
                    kind: ErrorKind::EvaluatorError,
                    message: format!(
                      "Tried to access field `{key_name}` on undefined or null variable `{recognized_name}`."
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
                      "Variable `{recognized_name}` is not an object and `{key_name}` is not available on it"
                    ),
                    source: None,
                  });
                }
              }
              pos += 2;
            }

            ExpressionToken::LeftBracket => {
              let (index_value, new_pos) = evaluate_expression_value(tokens, pos + 1, context)?;
              if tokens[new_pos] != ExpressionToken::RightBracket {
                return Err(Error {
                  kind: ErrorKind::EvaluatorError,
                  message: "Indexing is not finished with right bracket".to_string(),
                  source: None,
                });
              };
              pos = new_pos + 1;
              match index_value {
                Value::Number(index_num) => {
                  let Some(index_int) = index_num.as_u64() else {
                    return Err(Error {
                      kind: ErrorKind::EvaluatorError,
                      message: format!(
                        "Number index should be an unsiged integer, found {index_num:?}"
                      ),
                      source: None,
                    });
                  };
                  match value_ref {
                    Value::Array(arr) => {
                      let Some(v_ref) = arr.get(index_int as usize) else {
                        return Err(Error {
                          kind: ErrorKind::EvaluatorError,
                          message: format!(
                            "Out of bound: index {}, array length: {}",
                            index_int,
                            arr.len()
                          ),
                          source: None,
                        });
                      };
                      value_ref = v_ref;
                      recognized_name = recognized_name + &format!("[{index_num}]");
                    }
                    _ => {
                      return Err(Error {
                        kind: ErrorKind::EvaluatorError,
                        message: "Number index can only be applied on array.".to_string(),
                        source: None,
                      });
                    }
                  }
                }
                Value::String(index_str) => {
                  match value_ref {
                    Value::Null => {
                      return Err(Error {
                        kind: ErrorKind::EvaluatorError,
                        message: format!(
                          "Tried to access field `{index_str}` on undefined or null variable `{recognized_name}`"
                        ),
                        source: None,
                      });
                    }

                    Value::Object(obj) => match obj.get(&index_str) {
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
                        message: "String index can only be applied on object.".to_string(),
                        source: None,
                      });
                    }
                  };
                  recognized_name = recognized_name + &format!("[\"{index_str}\"]");
                }
                _ => {
                  return Err(Error {
                    kind: ErrorKind::EvaluatorError,
                    message: "Invalid index type.".to_string(),
                    source: None,
                  });
                }
              }
            }

            _ => break,
          }
        }
        return Ok((value_ref.to_owned(), pos));
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
          message: "Expect a value token, but not found".to_string(),
          source: None,
        });
      }
    }
  }
  Err(Error {
    kind: ErrorKind::EvaluatorError,
    message: "Not implemented".to_string(),
    source: None,
  })
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
        message: "String decode error".to_string(),
        source: Some(Box::new(e)),
      });
    }
  };
  match context.get_value(refs) {
    Some(r) => Ok(r.clone()),
    None => Ok(Value::Null),
  }
}

fn evaluate_number(numc: &[u8]) -> Result<Value> {
  let nums = str::from_utf8(numc).unwrap();

  if !numc.contains(&b'.') {
    let Ok(val): std::result::Result<i64, _> = str::parse(nums) else {
      return Err(Error {
        kind: ErrorKind::EvaluatorError,
        message: format!("Failed to parse number: {nums}"),
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
        message: format!("Failed to parse number: {nums}"),
        source: None,
      });
    };
    Ok(Value::Number(serde_json::Number::from_f64(val).unwrap()))
  }
}

fn evaluate_string(strc: &[u8]) -> Result<Value> {
  let str_val = match str::from_utf8(&strc[1..strc.len() - 1]) {
    Ok(s) => s,
    Err(e) => {
      return Err(Error {
        kind: ErrorKind::EvaluatorError,
        message: "Failed to decode string literal in expression.".to_string(),
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
    Value::Number(n) => Some(format!("{n}")),
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
  let str_a = cast_as_string(a);
  let str_b = cast_as_string(b);
  if str_a.is_some() && str_b.is_some() {
    return Ok(Value::String(format!(
      "{}{}",
      str_a.unwrap(),
      str_b.unwrap()
    )));
  }
  Err(Error {
    kind: ErrorKind::EvaluatorError,
    message: format!("Failed to perform plus operator on {a:?} and {b:?}."),
    source: None,
  })
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
  Err(Error {
    kind: ErrorKind::EvaluatorError,
    message: format!("Failed to perform minus operator on {a:?} and {b:?}."),
    source: None,
  })
}

fn handle_times_operator(a: &Value, b: &Value) -> Result<Value> {
  let int_a = cast_as_i64(a);
  let int_b = cast_as_i64(b);
  if int_a.is_some() && int_b.is_some() {
    return Ok(Value::Number(
      serde_json::Number::from_i128((int_a.unwrap() * int_b.unwrap()).into()).unwrap(),
    ));
  }
  let num_a = cast_as_f64(a);
  let num_b = cast_as_f64(b);
  if num_a.is_some() && num_b.is_some() {
    return Ok(Value::Number(
      serde_json::Number::from_f64(num_a.unwrap() * num_b.unwrap()).unwrap(),
    ));
  }
  Err(Error {
    kind: ErrorKind::EvaluatorError,
    message: format!("Failed to perform times operator on {a:?} and {b:?}."),
    source: None,
  })
}

fn handle_divide_operator(a: &Value, b: &Value) -> Result<Value> {
  let num_a = cast_as_f64(a);
  let num_b = cast_as_f64(b);
  if num_a.is_some() && num_b.is_some() {
    return Ok(Value::Number(
      serde_json::Number::from_f64(num_a.unwrap() / num_b.unwrap()).unwrap(),
    ));
  }
  Err(Error {
    kind: ErrorKind::EvaluatorError,
    message: format!("Failed to perform times operator on {a:?} and {b:?}."),
    source: None,
  })
}
#[cfg(test)]
mod tests;
