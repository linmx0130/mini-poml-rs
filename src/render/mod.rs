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
        let name = tag_node.name;
        // TODO evaluate attribute values
        let attribute_values = Vec::new();
        let mut children_result = Vec::new();
        for child in tag_node.children.iter() {
          children_result.push(self.render(child)?);
        }
        Ok(
          self
            .tag_renderer
            .render_tag(name, &attribute_values, children_result),
        )
      }
      PomlNode::Text(text) => self.render_text(text),
    }
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
      _ => {
        format!("{:?}", value)
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::PomlParser;
  use serde_json::json;
  use std::collections::HashMap;
  use tag_renderer::TestTagRenderer;

  #[test]
  fn test_render_content() {
    let doc = r#"
            <poml syntax="markdown">
                <p> Hello, {{name}}! </p>
            </poml>
        "#;
    let mut variables = HashMap::new();
    variables.insert("name".to_owned(), json!("world"));
    let context = render_context::RenderContext::from_iter(variables);
    let mut renderer = Renderer {
      context: context,
      tag_renderer: TestTagRenderer {},
    };

    let mut parser = PomlParser::from_str(doc);
    let node = parser.parse_as_node().unwrap();
    let output = renderer.render(&PomlNode::Tag(node)).unwrap();
    assert!(output.contains("Hello, world!"));
  }
}
