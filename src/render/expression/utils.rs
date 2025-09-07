/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use serde_json::Value;

pub fn is_false_json_value(value: &Value) -> bool {
  match value {
    Value::Bool(b) => !b,
    Value::Null => true,
    Value::Number(n) => match n.as_f64() {
      Some(v) => v == 0.0,
      None => true,
    },
    Value::String(s) => s.is_empty(),
    _ => false,
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde_json::json;

  #[test]
  pub fn test_is_false_json_value() {
    assert!(is_false_json_value(&json!(0)));
    assert!(is_false_json_value(&json!(false)));
    assert!(is_false_json_value(&json!("")));
    assert!(is_false_json_value(&json!(None::<i32>)));
    assert!(!is_false_json_value(&json!("0")));
    assert!(!is_false_json_value(&json!([])));
    assert!(!is_false_json_value(&json!({})));
  }
}
