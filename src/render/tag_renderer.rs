/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::error::{Error, ErrorKind, Result};
use crate::{PomlNode, PomlTagNode};

pub trait TagRenderer {
  fn render_tag(
    &self,
    tag: &PomlTagNode,
    attribute_values: &Vec<(String, String)>,
    children_result: Vec<String>,
  ) -> Result<String>;
}

/**
 * The tag render that renders nothing except dumping the
 * name, attributes and children it receives.
 */
pub struct TestTagRenderer {}

impl TagRenderer for TestTagRenderer {
  fn render_tag(
    &self,
    tag: &PomlTagNode,
    attribute_values: &Vec<(String, String)>,
    children_result: Vec<String>,
  ) -> Result<String> {
    let mut answer = String::new();
    answer += &format!("Name: {}\n", tag.name);
    for (key, value) in attribute_values {
      answer += &format!("  - {}: {}\n", key, value);
    }
    answer += "=====\n";
    for c in children_result {
      answer += &format!("{}\n", c);
    }
    answer += "=====\n";
    Ok(answer)
  }
}

/**
 * The default renderer to render markdown content.
 */

pub struct MarkdownTagRenderer {}

impl TagRenderer for MarkdownTagRenderer {
  fn render_tag(
    &self,
    tag: &PomlTagNode,
    attribute_values: &Vec<(String, String)>,
    children_result: Vec<String>,
  ) -> Result<String> {
    match tag.name {
      "poml" => self.render_poml_tag(tag, children_result),
      "p" => Ok(self.render_p_tag(children_result)),
      "b" => Ok(self.render_bold_tag(children_result)),
      "i" => Ok(self.render_italic_tag(children_result)),
      "meta" => Ok("".to_owned()),
      _ => Err(Error {
        kind: ErrorKind::RendererError,
        message: format!("Unknown tag: <{}>", tag.name),
        source: None,
      }),
    }
  }
}

impl MarkdownTagRenderer {
  fn render_poml_tag(
    &self,
    poml_tag: &PomlTagNode,
    children_result: Vec<String>,
  ) -> Result<String> {
    let children_tags = &poml_tag.children;
    if children_tags.len() != children_result.len() {
      return Err(Error {
        kind: ErrorKind::RendererError,
        message: format!("Missing children result in rendering <poml>."),
        source: None,
      });
    }

    let mut answer = String::new();
    for i in 0..children_tags.len() {
      if children_tags[i] != PomlNode::Whitespace {
        answer += &children_result[i];
      }
    }

    Ok(answer)
  }

  fn render_p_tag(&self, children_result: Vec<String>) -> String {
    format!("{}\n\n", children_result.join(""))
  }

  fn render_bold_tag(&self, children_result: Vec<String>) -> String {
    format!("**{}**", children_result.join(""))
  }

  fn render_italic_tag(&self, children_result: Vec<String>) -> String {
    format!("*{}*", children_result.join(""))
  }
}
