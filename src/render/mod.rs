/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod render_context;
pub mod tag_renderer;

use crate::error::{Error, ErrorKind, Result};
use crate::{PomlNode, PomlTagNode};
use serde_json::Value;

pub struct Renderer<T>
where
  T: tag_renderer::TagRenderer,
{
  pub context: render_context::RenderContext,
  pub tag_renderer: T,
}

impl<T> Renderer<T>
where
  T: tag_renderer::TagRenderer,
{
  pub fn render(&mut self, node: &PomlNode) -> Result<String> {
    match node {
      PomlNode::Tag(tag_node) => {
        let mut attribute_values: Vec<(String, String)> = Vec::new();
        for (key, value_raw) in tag_node.attributes.iter() {
          let value = self.render_text(&value_raw[1..value_raw.len() - 1])?;
          if key == &"if" {
            if self.is_false_value(&value) {
              return Ok("".to_owned());
            }
          }
          attribute_values.push((key.to_string(), value));
        }
        let mut children_result = Vec::new();
        for child in tag_node.children.iter() {
          children_result.push(self.render(child)?);
        }

        if tag_node.name == "let" {
          self.process_let_node(attribute_values)
        } else {
          Ok(
            self
              .tag_renderer
              .render_tag(tag_node, &attribute_values, children_result)?,
          )
        }
      }
      PomlNode::Text(text) => self.render_text(text),
      PomlNode::Whitespace => Ok(" ".to_owned()),
    }
  }

  fn process_let_node(&mut self, attribute_values: Vec<(String, String)>) -> Result<String> {
    /* TODO
     * 1. Support type attribute
     * 2. Support src attribute
     * 3. Support the case where name / value is missing
     * 4. Support using the content as value
     */
    let Some((_, name)) = attribute_values.iter().find(|v| v.0 == "name") else {
      return Err(Error {
        kind: ErrorKind::RendererError,
        message: format!("`name` attribute not found on <let>."),
        source: None,
      });
    };

    let Some((_, value)) = attribute_values.iter().find(|v| v.0 == "value") else {
      return Err(Error {
        kind: ErrorKind::RendererError,
        message: format!("`value` attribute not found on <let>."),
        source: None,
      });
    };

    self.context.set_value(name, value);
    Ok("".to_owned())
  }

  /**
   * Render the text node by replacing all expressions with the
   * variable values
   */
  fn render_text(&self, text: &str) -> Result<String> {
    let p = text.as_bytes();
    let mut answer_buf = Vec::with_capacity(p.len());
    let mut pos = 0;
    while pos < p.len() {
      if pos + 1 < p.len() && p[pos] == b'{' && p[pos + 1] == b'{' {
        let expression_start = pos + 2;
        let expression_end = {
          let mut t = expression_start;
          let mut expression_found = false;
          while t + 2 < p.len() {
            if p[t + 1] == b'}' && p[t + 2] == b'}' {
              expression_found = true;
              break;
            } else {
              t += 1;
            }
          }
          if !expression_found {
            return Err(Error {
              kind: ErrorKind::RendererError,
              // TODO add line/col position for the error message.
              message: format!("Expression end not found in text content."),
              source: None,
            });
          }
          t + 1
        };
        pos = expression_end + 2;
        let expression = str::from_utf8(&p[expression_start..expression_end]).unwrap();
        let result = self.context.evaluate(expression)?;
        let result_str = self.render_value(result);
        answer_buf.extend(result_str.as_bytes());
      } else if p[pos] == b'#' {
        // escaping process
      } else {
        answer_buf.push(p[pos]);
        pos += 1;
      }
    }
    let answer = String::from_utf8(answer_buf).unwrap();
    Ok(answer)
  }

  fn render_value(&self, value: Value) -> String {
    match value {
      Value::String(s) => s,
      Value::Null => "null".to_owned(),
      _ => {
        format!("{:?}", value)
      }
    }
  }

  fn is_false_value(&self, value: &str) -> bool {
    let val = value.trim();
    match val {
      "0" | "false" | "" | "null" | "NaN" => true,
      _ => false,
    }
  }
}

#[cfg(test)]
mod tests;
