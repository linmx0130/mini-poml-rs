/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use super::TagRenderer;
use crate::error::{Error, ErrorKind, Result};
use crate::render::utils;
use crate::{PomlNode, PomlTagNode};
/**
 * The default renderer to render markdown content.
 */
#[derive(Clone)]
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
      "h" => Ok(self.render_header_tag(children_result)),
      "section" => Ok(self.render_section_tag(children_result)),
      "cp" => self.render_captioned_paragraph_tag(attribute_values, children_result),
      "role" => Ok(self.render_intention_block_tag("Role", children_result)),
      "task" => Ok(self.render_intention_block_tag("Task", children_result)),
      "output-format" => Ok(self.render_intention_block_tag("Output Format", children_result)),
      "stepwise-instructions" => {
        Ok(self.render_intention_block_tag("Stepwise Instructions", children_result))
      }
      "meta" => Ok("".to_owned()),
      "item" => Ok(self.render_item_tag(children_result)),
      "list" => self.render_list_tag(tag, attribute_values, children_result),
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

  fn render_header_tag(&self, children_result: Vec<String>) -> String {
    format!("# {}\n\n", children_result.join(""))
  }

  fn render_section_tag(&self, children_result: Vec<String>) -> String {
    let mut answer = String::new();
    for child_text in children_result.iter() {
      if child_text.starts_with("#") {
        answer += &format!("#{}", child_text);
      } else {
        answer += child_text;
      }
    }
    answer
  }

  fn render_captioned_paragraph_tag(
    &self,
    attribute_values: &Vec<(String, String)>,
    children_result: Vec<String>,
  ) -> Result<String> {
    let Some((_, caption)) = attribute_values.iter().find(|v| v.0 == "caption") else {
      return Err(Error {
        kind: ErrorKind::RendererError,
        message: format!("Missing `caption` attribute for the <cp> tag."),
        source: None,
      });
    };
    let mut answer = format!("# {}\n\n", caption);

    for child_text in children_result.iter() {
      if child_text.starts_with("#") {
        answer += &format!("#{}", child_text);
      } else {
        answer += child_text;
      }
    }
    Ok(answer)
  }

  fn render_item_tag(&self, children_result: Vec<String>) -> String {
    let raw_content = children_result.join("");
    let lines: Vec<&str> = raw_content.split('\n').collect();
    let mut new_content = lines.get(0).map_or("", |v| v).to_string();
    for l in &lines[1..lines.len()] {
      new_content += "\n\t";
      new_content += l;
    }
    new_content += "\n";
    new_content
  }

  fn render_list_tag(
    &self,
    tag: &PomlTagNode,
    attribute_values: &Vec<(String, String)>,
    children_result: Vec<String>,
  ) -> Result<String> {
    let children_tags = &tag.children;
    if children_tags.len() != children_result.len() {
      return Err(Error {
        kind: ErrorKind::RendererError,
        message: format!("Missing children result in rendering <list>."),
        source: None,
      });
    }
    let list_style = match attribute_values.iter().find(|v| v.0 == "listStyle") {
      Some((_, v)) => v,
      None => "dash",
    };
    let mut answer = String::new();
    let mut item_counter = 0;
    for i in 0..children_tags.len() {
      let PomlNode::Tag(ref tag_node) = children_tags[i] else {
        continue;
      };
      if tag_node.name != "item" {
        // skip non item children
        continue;
      }
      for l in children_result[i].split("\n") {
        if l.len() == 0 {
          answer += "\n";
        } else if l.starts_with("\t") {
          answer += l;
          answer += "\n";
        } else {
          item_counter += 1;
          let item_mark = match list_style {
            "dash" => "- ".to_owned(),
            "star" => "* ".to_owned(),
            "plus" => "+ ".to_owned(),
            "decimal" => format!("{}. ", item_counter),
            _ => {
              return Err(Error {
                kind: ErrorKind::RendererError,
                message: format!("Unknown list style: {}", list_style),
                source: None,
              });
            }
          };

          answer += &item_mark;
          answer += l;
          answer += "\n";
        }
      }
    }
    Ok(answer)
  }
}
