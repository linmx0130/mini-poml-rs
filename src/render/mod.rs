/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod render_context;
pub mod tag_renderer;
pub(crate) mod utils;

use crate::error::{Error, ErrorKind, Result};
use crate::{PomlNode, PomlParser, PomlTagNode};
use serde_json::Value;

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
        for (key, value_raw) in tag_node.attributes.iter() {
          let value = self.render_text(&value_raw[1..value_raw.len() - 1])?;
          if key == &"if" {
            if utils::is_false_value(&value) {
              return Ok("".to_owned());
            }
          }
          attribute_values.push((key.to_string(), value));
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
          self.process_let_node(attribute_values)
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

  fn process_include_node(&mut self, attribute_values: Vec<(String, String)>) -> Result<String> {
    use std::io::Read;
    let Some((_, src)) = attribute_values.iter().find(|v| v.0 == "src") else {
      return Err(Error {
        kind: ErrorKind::RendererError,
        message: format!("`src` attribute not found on <include>."),
        source: None,
      });
    };

    let mut file_content_buf = String::new();
    let content = if self.context.file_mapping.contains_key(src) {
      self.context.file_mapping.get(src).unwrap()
    } else {
      let mut file = match std::fs::File::open(src) {
        Ok(f) => f,
        Err(e) => {
          return Err(Error {
            kind: ErrorKind::RendererError,
            message: format!("Failed to open file included: {}", src),
            source: Some(Box::new(e)),
          });
        }
      };
      match file.read_to_string(&mut file_content_buf) {
        Ok(_) => {}
        Err(e) => {
          return Err(Error {
            kind: ErrorKind::RendererError,
            message: format!("Failed to read file included: {}", src),
            source: Some(Box::new(e)),
          });
        }
      };
      &file_content_buf
    };

    let new_context = self.context.clone();
    let new_tag_renderer = self.tag_renderer.clone();
    let parser = PomlParser::from_str(&content);
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
      Value::Null => "null".to_owned(),
      _ => {
        format!("{:?}", value)
      }
    }
  }
}

#[cfg(test)]
mod tests;
