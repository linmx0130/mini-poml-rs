/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use serde_json::Value;

pub fn cast_as_i64(v: &Value) -> Option<i64> {
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

pub fn cast_as_i64_pair(v1: &Value, v2: &Value) -> Option<(i64, i64)> {
  let a1 = cast_as_i64(v1)?;
  let a2 = cast_as_i64(v2)?;
  Some((a1, a2))
}

pub fn cast_as_f64(v: &Value) -> Option<f64> {
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

pub fn cast_as_f64_pair(v1: &Value, v2: &Value) -> Option<(f64, f64)> {
  let a1 = cast_as_f64(v1)?;
  let a2 = cast_as_f64(v2)?;
  Some((a1, a2))
}

pub fn cast_as_string(v: &Value) -> Option<String> {
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
