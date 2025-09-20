/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod error;
pub mod parser;
pub mod render;

use parser::PomlParser;
use serde_json::Value;

/**
 * Data structure that represents a node in POML document.
 */
#[derive(Debug, PartialEq)]
pub enum PomlNode<'a> {
  /** A tag node. */
  Tag(PomlTagNode<'a>),
  /** Pure text content */
  Text(&'a str, PomlNodePosition),
  /** Whitespace content, which could be ignore but plays the role of separators. */
  Whitespace(PomlNodePosition),
}

impl<'a> PomlNode<'a> {
  pub fn is_tag(&self) -> bool {
    matches!(self, PomlNode::Tag(_))
  }

  pub fn is_whitespace(&self) -> bool {
    matches!(self, PomlNode::Whitespace(_))
  }
}

/**
 * Original position of a node in the original document.
 */
#[derive(Debug, PartialEq)]
pub struct PomlNodePosition {
  pub start: usize,
  pub end: usize,
}

/**
 * Data structure to represent a POML Tag Node.
 */
#[derive(Debug, PartialEq)]
pub struct PomlTagNode<'a> {
  pub name: &'a str,
  pub attributes: Vec<(&'a str, &'a str)>,
  pub children: Vec<PomlNode<'a>>,
  pub original_pos: PomlNodePosition,
}

/**
 * Render POML files into Markdown format.
 */
pub type MarkdownPomlRenderer<'a> = render::Renderer<'a, render::tag_renderer::MarkdownTagRenderer>;

impl<'a> MarkdownPomlRenderer<'a> {
  /**
   * Create a Markdown POML Render instance with the POML document and a context.
   */
  pub fn create_from_doc_and_context(
    doc: &'a str,
    context: render::render_context::RenderContext,
  ) -> Self {
    let parser = PomlParser::from_poml_str(doc);
    render::Renderer {
      parser,
      context,
      tag_renderer: render::tag_renderer::MarkdownTagRenderer {},
    }
  }

  /**
   * Create a Markdown POML Render instance with the POML document and
   * variables to be used for rendering.
   */
  pub fn create_from_doc_and_variables(
    doc: &'a str,
    variables: impl IntoIterator<Item = (String, Value)>,
  ) -> Self {
    let context = render::render_context::RenderContext::from_iter(variables);
    Self::create_from_doc_and_context(doc, context)
  }
}
