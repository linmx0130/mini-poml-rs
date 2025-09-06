/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod expression;
pub mod render_context;
pub mod tag_renderer;
pub(crate) mod utils;

use crate::error::{Error, ErrorKind, Result};
use crate::{PomlNode, PomlParser, PomlTagNode};
use serde_json::{Value, json};

pub struct Renderer<'a, T>
where
  T: tag_renderer::TagRenderer,
{
  pub parser: PomlParser<'a>,
  pub context: render_context::RenderContext,
  pub tag_renderer: T,
}

impl<'a, T> Renderer<'a, T>
where
  T: tag_renderer::TagRenderer,
{
  pub fn render(&mut self) -> Result<String> {
    let node = self.parser.parse_as_node()?;
    self.render_impl(&PomlNode::Tag(node))
  }

  pub(crate) fn render_impl(&mut self, node: &PomlNode) -> Result<String> {
    match node {
      PomlNode::Tag(tag_node) => {
        let mut attribute_values: Vec<(String, String)> = Vec::new();
        let mut for_loop_attribute: Option<&str> = None;
        let mut if_attribute_present = false;
        for (key, value_raw) in tag_node.attributes.iter() {
          let value = self.render_text(&value_raw[1..value_raw.len() - 1])?;
          if key == &"if" {
            if_attribute_present = true;
          }
          if key == &"for" {
            for_loop_attribute = Some(&value_raw[1..value_raw.len() - 1]);
          } else {
            attribute_values.push((key.to_string(), value));
          }
        }
        if if_attribute_present && for_loop_attribute.is_some() {
          return Err(Error {
            kind: ErrorKind::RendererError,
            message: format!(
              "Control flow attributes `if` and `for` on the same node is not supported!"
            ),
            source: None,
          });
        }

        // Process for loop
        if let Some(for_loop_instruction) = for_loop_attribute {
          let for_loop_components: Vec<&str> = for_loop_instruction.split(" in ").collect();
          if for_loop_components.len() != 2 {
            return Err(Error {
              kind: ErrorKind::RendererError,
              message: format!("Invalid for-loop attribute value: {}", for_loop_instruction),
              source: None,
            });
          }
          let for_item_name = for_loop_components[0].trim();
          let for_range_value = self.context.evaluate(for_loop_components[1].trim())?;
          let Value::Array(for_range) = for_range_value else {
            return Err(Error {
              kind: ErrorKind::RendererError,
              message: format!(
                "For loop range is not an array: {}",
                for_loop_components[1].trim()
              ),
              source: None,
            });
          };

          self.context.push_scope();
          let mut answer = String::new();
          for (item_idx, item_value) in for_range.iter().enumerate() {
            self.context.set_value(for_item_name, item_value.clone());
            let loop_variable = json!({
                "index": item_idx,
                "length": for_range.len(),
                "first": item_idx == 0,
                "last": item_idx + 1 == for_range.len()
            });
            self.context.set_value("loop", loop_variable);
            let item_node_result =
              self.process_tag_node_without_for(&tag_node, attribute_values.clone())?;
            answer += &item_node_result;
          }
          self.context.pop_scope();
          return Ok(answer);
        } else {
          self.process_tag_node_without_for(&tag_node, attribute_values)
        }
      }
      PomlNode::Text(text) => self.render_text(text),
      PomlNode::Whitespace => Ok(" ".to_owned()),
    }
  }

  /**
   * The loop attribute `for` should be processed before this function is called.
   */
  fn process_tag_node_without_for(
    &mut self,
    tag_node: &PomlTagNode,
    attribute_values: Vec<(String, String)>,
  ) -> Result<String> {
    for (key, value) in &attribute_values {
      if key == &"if" {
        if utils::is_false_value(&value) {
          return Ok("".to_owned());
        }
      }
    }

    let mut children_result = Vec::new();
    if tag_node.children.len() > 0 {
      self.context.push_scope();
      for child in tag_node.children.iter() {
        children_result.push(self.render_impl(child)?);
      }
      self.context.pop_scope();
    }

    if tag_node.name == "let" {
      self.process_let_node(attribute_values, children_result)
    } else if tag_node.name == "include" {
      self.process_include_node(attribute_values)
    } else {
      Ok(self.tag_renderer.render_tag(
        tag_node,
        &attribute_values,
        children_result,
        self.parser.buf,
      )?)
    }
  }

  fn process_let_node(
    &mut self,
    attribute_values: Vec<(String, String)>,
    children_result: Vec<String>,
  ) -> Result<String> {
    let name = match attribute_values.iter().find(|v| v.0 == "name") {
      Some((_, value)) => Some(value),
      None => None,
    };

    // Check whether more than one source of value is provided
    let children_value = if children_result.len() > 0 {
      Some(children_result.join(""))
    } else {
      None
    };

    let attribute_value = match attribute_values.iter().find(|v| v.0 == "value") {
      Some((_, value)) => Some(value.to_string()),
      None => None,
    };

    let src_value = match attribute_values.iter().find(|v| v.0 == "src") {
      Some((_, src)) => {
        let file_content_buf = self.context.read_file_content(src)?;
        Some(file_content_buf)
      }
      None => None,
    };
    let mut value_count = 0;
    if children_value.is_some() {
      value_count += 1;
    }
    if src_value.is_some() {
      value_count += 1;
    }
    if attribute_value.is_some() {
      value_count += 1;
    }

    let type_value = match attribute_values.iter().find(|v| v.0 == "type") {
      Some((_, v)) => v,
      None => {
        if src_value.is_some() {
          "object"
        } else {
          "string"
        }
      }
    };

    let value: String = match value_count {
      0 => {
        return Err(Error {
          kind: ErrorKind::RendererError,
          message: format!("No value is provided for the <let> node"),
          source: None,
        });
      }
      1 => {
        if children_value.is_some() {
          children_value.unwrap()
        } else if src_value.is_some() {
          src_value.unwrap()
        } else {
          attribute_value.unwrap()
        }
      }
      _ => {
        return Err(Error {
          kind: ErrorKind::RendererError,
          message: format!("More than one value is provided for the <let> node."),
          source: None,
        });
      }
    };

    let Some(name) = name else {
      let Ok(Value::Object(value_obj)) = serde_json::from_str(&value) else {
        return Err(Error {
          kind: ErrorKind::RendererError,
          message: format!("Only object value can be used to set context variables"),
          source: None,
        });
      };
      for (key, value) in value_obj.iter() {
        self.context.set_value(key, value.clone());
      }
      return Ok("".to_owned());
    };

    if type_value != "string" {
      match type_value {
        "integer" => {
          let int_val: i64 = match str::parse(&value) {
            Ok(v) => v,
            Err(e) => {
              return Err(Error {
                kind: ErrorKind::RendererError,
                message: format!("Failed to convert value to integer {}", value),
                source: Some(Box::new(e)),
              });
            }
          };
          self.context.set_value(
            name,
            Value::Number(serde_json::Number::from_i128(int_val.into()).unwrap()),
          );
        }
        "number" => {
          if value.contains('.') {
            let fval: f64 = match str::parse(&value) {
              Ok(v) => v,
              Err(e) => {
                return Err(Error {
                  kind: ErrorKind::RendererError,
                  message: format!("Failed to convert value to number {}", value),
                  source: Some(Box::new(e)),
                });
              }
            };
            self.context.set_value(
              name,
              Value::Number(serde_json::Number::from_f64(fval.into()).unwrap()),
            );
          } else {
            let int_val: i64 = match str::parse(&value) {
              Ok(v) => v,
              Err(e) => {
                return Err(Error {
                  kind: ErrorKind::RendererError,
                  message: format!("Failed to convert value to number {}", value),
                  source: Some(Box::new(e)),
                });
              }
            };
            self.context.set_value(
              name,
              Value::Number(serde_json::Number::from_i128(int_val.into()).unwrap()),
            );
          }
        }
        "boolean" => {
          let bool_val = !utils::is_false_value(&value);
          self.context.set_value(name, Value::Bool(bool_val));
        }
        "array" => {
          match serde_json::from_str(&value) {
            Ok(Value::Array(value_arr)) => {
              self.context.set_value(name, Value::Array(value_arr));
            }
            _ => {
              return Err(Error {
                kind: ErrorKind::RendererError,
                message: format!("Failed to parse value to array: {}", value),
                source: None,
              });
            }
          };
        }
        "object" => {
          match serde_json::from_str(&value) {
            Ok(Value::Object(value_obj)) => {
              self.context.set_value(name, Value::Object(value_obj));
            }
            _ => {
              return Err(Error {
                kind: ErrorKind::RendererError,
                message: format!("Failed to parse value to object: {}", value),
                source: None,
              });
            }
          };
        }
        _ => {
          return Err(Error {
            kind: ErrorKind::RendererError,
            message: format!("Unknown type for varaible: {}", type_value),
            source: None,
          });
        }
      }
    } else {
      self.context.set_value(name, Value::String(value));
    }
    Ok("".to_owned())
  }

  fn process_include_node(&mut self, attribute_values: Vec<(String, String)>) -> Result<String> {
    let Some((_, src)) = attribute_values.iter().find(|v| v.0 == "src") else {
      return Err(Error {
        kind: ErrorKind::RendererError,
        message: format!("`src` attribute not found on <include>."),
        source: None,
      });
    };

    let file_content_buf = self.context.read_file_content(src)?;
    let new_context = self.context.clone();
    let new_tag_renderer = self.tag_renderer.clone();
    let parser = PomlParser::from_str(&file_content_buf);
    let mut renderer = Renderer {
      context: new_context,
      tag_renderer: new_tag_renderer,
      parser,
    };
    renderer.render()
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
        let escaping_mapping = [
          ("#quot;", b'"'),
          ("#apos;", b'\''),
          ("#amp;", b'&'),
          ("#lt;", b'<'),
          ("#gt;", b'>'),
          ("#hash;", b'#'),
          ("#lbrace;", b'{'),
          ("#rbrace;", b'}'),
        ];
        let mut escaped = false;
        for (escaping_pattern, escaping_target) in escaping_mapping {
          if utils::buf_match_str(p, pos, escaping_pattern) {
            escaped = true;
            answer_buf.push(escaping_target);
            pos = pos + escaping_pattern.len();
            break;
          }
        }
        if !escaped {
          answer_buf.push(p[pos]);
          pos += 1;
        }
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
      Value::Number(ref num) => {
        if num.is_i64() {
          format!("{}", num.as_i64().unwrap())
        } else if num.is_f64() {
          format!("{}", num.as_f64().unwrap())
        } else {
          "NaN".to_owned()
        }
      }
      Value::Bool(b) => {
        format!("{}", b)
      }
      Value::Null => "null".to_owned(),
      _ => {
        format!("{:?}", value)
      }
    }
  }
}

#[cfg(test)]
mod tests;
