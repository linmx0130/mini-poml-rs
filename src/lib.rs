/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod error;
pub mod parser;
pub mod render;

pub use parser::PomlParser;
use serde_json::Value;

#[derive(Debug, PartialEq)]
pub enum PomlNode<'a> {
  Tag(PomlTagNode<'a>),
  Text(&'a str),
  Whitespace,
}

impl<'a> PomlNode<'a> {
  pub fn is_tag(&self) -> bool {
    match self {
      PomlNode::Tag(_) => true,
      _ => false,
    }
  }
}

#[derive(Debug, PartialEq)]
pub struct PomlTagNode<'a> {
  pub name: &'a str,
  pub attributes: Vec<(&'a str, &'a str)>,
  pub children: Vec<PomlNode<'a>>,
  pub original_start_pos: usize,
  pub original_end_pos: usize,
}

pub type MarkdownPomlRenderer<'a> = render::Renderer<'a, render::tag_renderer::MarkdownTagRenderer>;

impl<'a> MarkdownPomlRenderer<'a> {
  pub fn create_from_doc_and_context(
    doc: &'a str,
    context: render::render_context::RenderContext,
  ) -> Self {
    let parser = PomlParser::from_str(&doc);
    render::Renderer {
      parser: parser,
      context: context,
      tag_renderer: render::tag_renderer::MarkdownTagRenderer {},
    }
  }
  pub fn create_from_doc_and_variables(
    doc: &'a str,
    variables: impl IntoIterator<Item = (String, Value)>,
  ) -> Self {
    let context = render::render_context::RenderContext::from_iter(variables);
    Self::create_from_doc_and_context(doc, context)
  }
}
