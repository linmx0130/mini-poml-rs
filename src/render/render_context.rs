/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::error::{Error, ErrorKind, Result};
use serde_json::{Map, Value, json};
use std::collections::HashMap;

/**
 * Contains the variables in the current scope.
 */
#[derive(Debug, Clone)]
pub struct Scope {
  variables: Map<String, Value>,
}

/**
 * Context to render the POML tags into desired output format
 */
#[derive(Debug, Clone)]
pub struct RenderContext {
  scope_layers: Vec<Scope>,
  pub(crate) file_mapping: HashMap<String, String>,
}

impl RenderContext {
  /**
   * Obtain the value of the given variable name in the current context.
   *
   * If the value doesn't exist, return `None`.
   */
  pub fn get_value(&self, name: &str) -> Option<&Value> {
    for i in (0..self.scope_layers.len()).rev() {
      match self.scope_layers[i].variables.get(name) {
        Some(v) => {
          return Some(v);
        }
        None => continue,
      }
    }
    None
  }

  /**
   * Set a value on the current scope. If there is no scope in the context,
   * nothing will happen.
   */
  pub fn set_value(&mut self, name: &str, value: Value) {
    if let Some(current_scope) = self.scope_layers.last_mut() {
      current_scope.variables.insert(name.to_string(), value);
    }
  }

  pub fn push_scope(&mut self) {
    self.scope_layers.push(Scope {
      variables: Map::new(),
    });
  }

  pub fn pop_scope(&mut self) {
    self.scope_layers.pop();
  }

  /**
   * Evaluate the value of an expression.
   */
  pub fn evaluate(&self, expression: &str) -> Result<Value> {
    // TODO: evaluate expression
    match self.get_value(expression) {
      Some(v) => Ok(v.to_owned()),
      None => Ok(Value::Null),
    }
  }
}

impl FromIterator<(String, Value)> for RenderContext {
  fn from_iter<I: IntoIterator<Item = (String, Value)>>(iter: I) -> Self {
    let base_scope = Scope {
      variables: Map::from_iter(iter),
    };

    RenderContext {
      scope_layers: vec![base_scope],
      file_mapping: HashMap::new(),
    }
  }
}

impl<'a> FromIterator<(&'a String, &'a Value)> for RenderContext {
  fn from_iter<I: IntoIterator<Item = (&'a String, &'a Value)>>(iter: I) -> Self {
    let mut vars = Map::new();
    for (k, v) in iter {
      vars.insert(k.to_owned(), v.to_owned());
    }

    let base_scope = Scope { variables: vars };

    RenderContext {
      scope_layers: vec![base_scope],
      file_mapping: HashMap::new(),
    }
  }
}

impl From<Map<String, Value>> for RenderContext {
  fn from(value: Map<String, Value>) -> Self {
    let base_scope = Scope { variables: value };
    RenderContext {
      scope_layers: vec![base_scope],
      file_mapping: HashMap::new(),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde_json::json;
  use std::collections::HashMap;

  #[test]
  fn test_creation_with_hashmap() {
    let mut variables = HashMap::new();
    variables.insert("a".to_owned(), json!(1));
    variables.insert("s".to_owned(), json!("s"));
    let context = RenderContext::from_iter(variables);
    assert_eq!(context.get_value("a"), Some(json!(1)).as_ref());
    assert_eq!(context.get_value("s"), Some(json!("s")).as_ref());
  }

  #[test]
  fn test_creation_with_hashmap_ref() {
    let mut variables = HashMap::new();
    variables.insert("a".to_owned(), json!(1));
    variables.insert("s".to_owned(), json!("s"));
    let context = RenderContext::from_iter(&variables);
    assert_eq!(context.get_value("a"), Some(json!(1)).as_ref());
    assert_eq!(context.get_value("s"), Some(json!("s")).as_ref());
  }

  #[test]
  fn test_creation_with_json_map() {
    let Value::Object(variables) = json!({
        "a": 1,
        "s": "s"
    }) else {
      panic!()
    };
    let context = RenderContext::from(variables);
    assert_eq!(context.get_value("a"), Some(json!(1)).as_ref());
    assert_eq!(context.get_value("s"), Some(json!("s")).as_ref());
  }
}
