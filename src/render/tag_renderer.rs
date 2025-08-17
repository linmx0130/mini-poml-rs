/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use super::utils;
use crate::error::{Error, ErrorKind, Result};
use crate::{PomlNode, PomlParser, PomlTagNode};

pub trait TagRenderer {
  fn render_tag(
    &self,
    tag: &PomlTagNode,
    attribute_values: &Vec<(String, String)>,
    children_result: Vec<String>,
    source_buf: &[u8],
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
    _source_buf: &[u8],
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
    source_buf: &[u8],
  ) -> Result<String> {
    match tag.name {
      "poml" => self.render_poml_tag(tag, children_result),
      "p" => Ok(self.render_p_tag(children_result)),
      "b" => Ok(self.render_bold_tag(children_result)),
      "i" => Ok(self.render_italic_tag(children_result)),
      "code" => Ok(self.render_code_tag(tag, attribute_values, source_buf)),
      "role" => Ok(self.render_intention_block_tag("Role", children_result)),
      "task" => Ok(self.render_intention_block_tag("Task", children_result)),
      "output-format" => Ok(self.render_intention_block_tag("Output Format", children_result)),
      "stepwise-instructions" => {
        Ok(self.render_intention_block_tag("Stepwise Instructions", children_result))
      }
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

  fn render_code_tag(
    &self,
    tag: &PomlTagNode,
    attribute_values: &Vec<(String, String)>,
    source_buf: &[u8],
  ) -> String {
    println!("{:?}", tag);
    let tag_code =
      str::from_utf8(&source_buf[tag.original_start_pos..tag.original_end_pos]).unwrap();
    let code_start = tag_code.find('>').unwrap() + 1;
    let code_end = tag_code.rfind("</").unwrap();
    let code_content = &tag_code[code_start..code_end];
    let mut inline = false;
    let mut lang: Option<&str> = None;
    for (attr_key, attr_value) in attribute_values.iter() {
      match attr_key.as_str() {
        "inline" => {
          if !utils::is_false_value(attr_value) {
            inline = true;
          }
        }
        "lang" => {
          lang = Some(attr_value);
        }
        _ => {}
      }
    }
    if inline {
      format!("`{}`", code_content)
    } else {
      let header = match lang {
        Some(l) => format!("```{}\n", l),
        None => format!("```\n"),
      };
      format!("{}{}\n```", header, code_content)
    }
  }

  fn render_intention_block_tag(&self, title: &str, children_result: Vec<String>) -> String {
    let mut answer = format!("# {}\n\n", title);
    for child_text in children_result.iter() {
      if child_text.starts_with("#") {
        answer += &format!("#{}", child_text);
      } else {
        answer += child_text;
      }
    }
    answer
  }
}
