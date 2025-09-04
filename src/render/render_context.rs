/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::error::{Error, ErrorKind, Result};
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::io::Read;

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
    super::expression::evaluate_expression(expression, self)
  }

  pub fn read_file_content(&self, filename: &str) -> Result<String> {
    if self.file_mapping.contains_key(filename) {
      Ok(self.file_mapping.get(filename).unwrap().to_string())
    } else {
      let mut file_content_buf = String::new();
      let mut file = match std::fs::File::open(filename) {
        Ok(f) => f,
        Err(e) => {
          return Err(Error {
            kind: ErrorKind::RendererError,
            message: format!("Failed to open file included: {}", filename),
            source: Some(Box::new(e)),
          });
        }
      };
      match file.read_to_string(&mut file_content_buf) {
        Ok(_) => {}
        Err(e) => {
          return Err(Error {
            kind: ErrorKind::RendererError,
            message: format!("Failed to read file included: {}", filename),
            source: Some(Box::new(e)),
          });
        }
      };
      Ok(file_content_buf)
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
