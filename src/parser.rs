/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::error::{Error, ErrorKind, Result};
use crate::{PomlNode, PomlNodePosition, PomlTagNode};

#[derive(Debug, PartialEq)]
pub enum PomlElementKind {
  Tag,
  Text,
  Whitespace,
}

#[derive(Debug, PartialEq)]
pub struct PomlElement {
  pub kind: PomlElementKind,
  pub start_pos: usize,
  pub end_pos: usize,
}

#[derive(Debug)]
pub struct PomlParser<'a> {
  pub buf: &'a [u8],
  pub pos: usize,
  pub line_end_pos: Vec<usize>,
}

impl<'a> PomlParser<'a> {
  pub fn from_poml_str(s: &'a str) -> PomlParser<'a> {
    let buf = s.as_bytes();
    let mut line_end_pos = Vec::new();
    let mut first_not_space = None;
    {
      for (pos, item) in buf.iter().enumerate() {
        if first_not_space.is_none() && !item.is_ascii_whitespace() {
          first_not_space = Some(pos);
        }
        if *item == b'\n' {
          line_end_pos.push(pos);
        }
      }
      if buf[buf.len() - 1] != b'\n' {
        line_end_pos.push(buf.len());
      }
    }

    PomlParser {
      buf,
      pos: first_not_space.unwrap_or(buf.len()),
      line_end_pos,
    }
  }

  pub fn parse_as_node(&mut self) -> Result<PomlTagNode<'a>> {
    let elements = self.parse_as_elements()?;
    let mut node_stack: Vec<PomlTagNode> = Vec::new();
    let mut added_poml_root = false;

    for element in elements.iter() {
      match element.kind {
        PomlElementKind::Text => {
          let last_node = match node_stack.last_mut() {
            Some(l) => l,
            None => {
              return Err(Error {
                kind: ErrorKind::ParserError,
                message: format!(
                  "Text appears at position {:?} without a node",
                  self.get_line_and_col_from_pos(element.start_pos)
                ),
                source: None,
              });
            }
          };
          let text = str::from_utf8(&self.buf[element.start_pos..element.end_pos]).unwrap();
          let text_node = PomlNode::Text(
            text,
            PomlNodePosition {
              start: element.start_pos,
              end: element.end_pos,
            },
          );
          last_node.children.push(text_node);
        }
        PomlElementKind::Whitespace => {
          let last_node = match node_stack.last_mut() {
            Some(l) => l,
            None => {
              return Err(Error {
                kind: ErrorKind::ParserError,
                message: format!(
                  "Whitespace appears at position {:?} without a node",
                  self.get_line_and_col_from_pos(element.start_pos)
                ),
                source: None,
              });
            }
          };
          let whitespace_node = PomlNode::Whitespace(PomlNodePosition {
            start: element.start_pos,
            end: element.end_pos,
          });
          last_node.children.push(whitespace_node);
        }
        PomlElementKind::Tag => {
          if self.is_self_close_tag_element(element) {
            let tag = self.create_tag_from_element(element)?;
            if node_stack.is_empty() {
              if tag.name != "poml" {
                node_stack.push(PomlTagNode {
                  name: "poml",
                  attributes: vec![],
                  children: vec![],
                  original_pos: PomlNodePosition {
                    start: element.start_pos,
                    end: element.end_pos,
                  },
                });
                added_poml_root = true;
              } else {
                return Err(Error {
                  kind: ErrorKind::ParserError,
                  message: "<poml> tag should not close itself.".to_string(),
                  source: None,
                });
              }
            }
            match node_stack.last_mut() {
              Some(l) => {
                l.children.push(PomlNode::Tag(tag));
              }
              None => {
                unreachable!()
              }
            }
          } else if self.is_close_tag_element(element) {
            // check tag name
            let mut node_to_close = match node_stack.pop() {
              Some(l) => l,
              None => {
                return Err(Error {
                  kind: ErrorKind::ParserError,
                  message: format!(
                    "Close tag appears without an open tag at position {:?}",
                    self.get_line_and_col_from_pos(element.start_pos)
                  ),
                  source: None,
                });
              }
            };
            let (tag_name, _) = self.consume_key_str(element.start_pos + 2);
            if tag_name != node_to_close.name {
              return Err(Error {
                kind: ErrorKind::ParserError,
                message: format!(
                  "Close tag of </{}> appears at position {:?}, but the open tag is <{}>",
                  tag_name,
                  self.get_line_and_col_from_pos(element.start_pos),
                  node_to_close.name
                ),
                source: None,
              });
            }
            node_to_close.original_pos.end = element.end_pos;
            match node_stack.last_mut() {
              Some(l) => {
                l.children.push(PomlNode::Tag(node_to_close));
              }
              None => {
                return Ok(node_to_close);
              }
            }
          } else {
            let tag = self.create_tag_from_element(element)?;
            if node_stack.is_empty() && tag.name != "poml" {
              node_stack.push(PomlTagNode {
                name: "poml",
                attributes: vec![],
                children: vec![],
                original_pos: PomlNodePosition {
                  start: 0,
                  end: self.buf.len(),
                },
              });
              added_poml_root = true;
            }
            node_stack.push(tag);
          }
        }
      }
    }

    if node_stack.len() == 1 && added_poml_root {
      Ok(node_stack.pop().unwrap())
    } else {
      Err(Error {
        kind: ErrorKind::ParserError,
        message: "Document has not finished at the end".to_owned(),
        source: None,
      })
    }
  }

  fn create_tag_from_element(&self, element: &PomlElement) -> Result<PomlTagNode<'a>> {
    let (tag_name, mut pos) = self.consume_key_str(element.start_pos + 1);
    let mut attributes: Vec<(&'a str, &'a str)> = Vec::new();
    loop {
      pos = self.consume_space(pos);
      if self.buf[pos].is_ascii_alphanumeric() {
        let (attribute_name, next_pos) = self.consume_key_str(pos);
        if attributes.iter().any(|v| v.0 == attribute_name) {
          return Err(Error {
            kind: ErrorKind::ParserError,
            message: format!(
              "Duplicate attribute key at position {:?}: {}",
              self.get_line_and_col_from_pos(pos),
              attribute_name
            ),
            source: None,
          });
        }
        pos = self.consume_space(next_pos);
        // Expect to see '='
        if self.buf[pos] != b'=' {
          return Err(Error {
            kind: ErrorKind::ParserError,
            message: format!(
              "Expect '=' for attribute declaration at position {:?}, but not found.",
              self.get_line_and_col_from_pos(pos)
            ),
            source: None,
          });
        }
        pos = self.consume_space(pos + 1);
        // Expect to see '"' as the start of a string literal
        if self.buf[pos] != b'"' {
          return Err(Error {
            kind: ErrorKind::ParserError,
            message: format!(
              "Expect '\"' for attribute value at position {:?}, but not found.",
              self.get_line_and_col_from_pos(pos)
            ),
            source: None,
          });
        }
        let (attribute_value, next_pos) = self.consume_value_str_literal(pos)?;
        attributes.push((attribute_name, attribute_value));
        pos = next_pos
      } else {
        break;
      }
    }

    Ok(PomlTagNode {
      name: tag_name,
      attributes,
      children: Vec::new(),
      original_pos: PomlNodePosition {
        start: element.start_pos,
        end: element.end_pos,
      },
    })
  }

  fn is_self_close_tag_element(&self, element: &PomlElement) -> bool {
    if element.kind == PomlElementKind::Tag {
      self.buf[element.end_pos - 2] == b'/'
    } else {
      false
    }
  }

  fn is_close_tag_element(&self, element: &PomlElement) -> bool {
    if element.kind == PomlElementKind::Tag {
      self.buf[element.start_pos + 1] == b'/'
    } else {
      false
    }
  }

  /**
   * Consume a key (tag name or attribute name) str.
   * Return the key str reference and the next position.
   */
  fn consume_key_str(&self, pos: usize) -> (&'a str, usize) {
    let buf = self.buf;
    let mut next_pos = pos;
    while next_pos < buf.len() {
      let c = char::from(buf[next_pos]);
      if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
        next_pos += 1
      } else {
        break;
      }
    }
    (str::from_utf8(&buf[pos..next_pos]).unwrap(), next_pos)
  }

  /**
   * Consume a value string literal.
   *
   * Return the value str reference and the next position after the ending quote.
   */
  fn consume_value_str_literal(&self, pos: usize) -> Result<(&'a str, usize)> {
    let buf = self.buf;
    if buf[pos] != b'"' {
      return Err(Error {
        kind: ErrorKind::ParserError,
        message: format!(
          "Expect to see '\"' as the start of a literal at position {:?}, but found {}",
          self.get_line_and_col_from_pos(pos),
          buf[pos]
        ),
        source: None,
      });
    }
    let mut next_pos = pos + 1;
    while next_pos < buf.len() {
      match buf[next_pos] {
        b'\\' => {
          next_pos += 2;
        }
        b'"' => break,
        _ => next_pos += 1,
      }
    }
    if buf[next_pos] == b'\"' {
      Ok((
        str::from_utf8(&buf[pos..next_pos + 1]).unwrap(),
        next_pos + 1,
      ))
    } else {
      Err(Error {
        kind: ErrorKind::ParserError,
        message: format!(
          "String literal has not reach an end at position {:?}",
          self.get_line_and_col_from_pos(next_pos)
        ),
        source: None,
      })
    }
  }

  /**
   * Consume ASCII spaces.
   * Return the first non-space character position at or after the `pos`
   */
  fn consume_space(&self, pos: usize) -> usize {
    let buf = self.buf;
    let mut next_pos = pos;
    while next_pos < buf.len() {
      let c = char::from(buf[next_pos]);
      if c.is_ascii_whitespace() {
        next_pos += 1;
      } else {
        break;
      }
    }
    next_pos
  }

  pub(crate) fn parse_as_elements(&mut self) -> Result<Vec<PomlElement>> {
    let mut elements = Vec::new();
    while let Some(e) = self.next_element()? {
      elements.push(e);
    }
    Ok(elements)
  }

  fn next_element(&mut self) -> Result<Option<PomlElement>> {
    if self.pos < self.buf.len() {
      let c = char::from(self.buf[self.pos]);
      match c {
        c if c.is_ascii_whitespace() => {
          let start_pos = self.pos;
          let end_pos = self.consume_space(self.pos);
          self.pos = end_pos;
          return Ok(Some(PomlElement {
            kind: PomlElementKind::Whitespace,
            start_pos,
            end_pos,
          }));
        }
        '<' => {
          let start_pos = self.pos;
          let end_pos = match self.seek_gt_char(start_pos + 1) {
            Some(p) => p,
            None => {
              return Err(Error {
                kind: ErrorKind::ParserError,
                message: format!("Tag starting from {start_pos} is not complete"),
                source: None,
              });
            }
          };
          self.pos = end_pos;
          return Ok(Some(PomlElement {
            kind: PomlElementKind::Tag,
            start_pos,
            end_pos,
          }));
        }
        _ => {
          let start_pos = self.pos;
          let end_pos = self.seek_end_of_text(start_pos + 1);
          self.pos = end_pos;
          return Ok(Some(PomlElement {
            kind: PomlElementKind::Text,
            start_pos,
            end_pos,
          }));
        }
      }
    }
    Ok(None)
  }

  fn seek_gt_char(&self, pos: usize) -> Option<usize> {
    let mut pos = pos;
    let mut in_string = false;
    while pos < self.buf.len() {
      match self.buf[pos] {
        b'>' => {
          if !in_string {
            return Some(pos + 1);
          }
        }
        b'"' => {
          in_string = !in_string;
        }
        b'\\' => {
          if in_string {
            // skip next character due to escape
            pos += 1;
          }
        }
        _ => {}
      }
      pos += 1;
    }
    None
  }

  fn seek_end_of_text(&self, pos: usize) -> usize {
    let mut pos = pos;
    while pos < self.buf.len() {
      match self.buf[pos] {
        b'<' | b'\r' | b'\n' => {
          return pos;
        }
        _ => {
          pos += 1;
        }
      }
    }
    pos
  }

  /**
   * Get the line and col number from the postion value for error message.
   *
   * All numbers are indexed from 0.
   */
  fn get_line_and_col_from_pos(&self, pos: usize) -> (usize, usize) {
    if pos >= self.buf.len() {
      return (self.line_end_pos.len(), 0);
    }
    let mut l = 0;
    let mut r = self.line_end_pos.len() - 1;
    while l < r {
      let mid = (l + r) / 2;
      if self.line_end_pos[mid] < pos {
        l = mid + 1;
      } else {
        r = mid;
      }
    }
    let line_number = l;
    let offset = if line_number != 0 {
      self.line_end_pos[line_number - 1]
    } else {
      0
    };
    let col = pos - offset;
    (line_number, col)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn tokenize_simple_poml() {
    let doc = r#"<poml syntax="markdown"><p> Hello, {{ name }}! </p></poml>"#;
    let mut parser = PomlParser::from_poml_str(doc);
    let elements = parser.parse_as_elements().unwrap();
    assert_eq!(elements.len(), 6);
    assert_eq!(elements[0].kind, PomlElementKind::Tag);
    assert_eq!(
      &doc[elements[0].start_pos..elements[0].end_pos],
      "<poml syntax=\"markdown\">"
    );
    assert_eq!(elements[1].kind, PomlElementKind::Tag);
    assert_eq!(&doc[elements[1].start_pos..elements[1].end_pos], "<p>");
    assert_eq!(elements[2].kind, PomlElementKind::Whitespace);
    assert_eq!(elements[3].kind, PomlElementKind::Text);
    assert_eq!(
      &doc[elements[3].start_pos..elements[3].end_pos],
      "Hello, {{ name }}! "
    );
    assert_eq!(elements[4].kind, PomlElementKind::Tag);
    assert_eq!(&doc[elements[4].start_pos..elements[4].end_pos], "</p>");
    assert_eq!(elements[4].kind, PomlElementKind::Tag);
    assert_eq!(&doc[elements[5].start_pos..elements[5].end_pos], "</poml>");
  }

  #[test]
  fn tokenize_tag_with_escape_in_attributes() {
    let doc = r#"<let name="foo" value=">bar\"" />"#;
    let mut parser = PomlParser::from_poml_str(doc);
    let elements = parser.parse_as_elements().unwrap();
    let element = &elements[0];
    assert_eq!(element.kind, PomlElementKind::Tag);
    assert_eq!(element.start_pos, 0);
    assert_eq!(element.end_pos, doc.len());
  }

  #[test]
  fn parse_as_node_simple_doc() {
    let doc = r#"
        <poml syntax="markdown">
            <p>Hello, {{ name }}!</p>
        </poml>
        "#;
    let mut parser = PomlParser::from_poml_str(doc);
    let node = parser.parse_as_node().unwrap();
    assert_eq!(node.name, "poml");
    assert_eq!(node.attributes.len(), 1);
    assert_eq!(
      node.attributes.first().unwrap(),
      &("syntax", "\"markdown\"")
    );
    let tag_children: Vec<&PomlNode> = node.children.iter().filter(|v| v.is_tag()).collect();
    assert_eq!(tag_children.len(), 1);
    let PomlNode::Tag(p_node) = tag_children.first().unwrap() else {
      panic!()
    };
    assert_eq!(p_node.name, "p");
    assert_eq!(
      p_node.children.first().unwrap(),
      &PomlNode::Text(
        "Hello, {{ name }}!",
        PomlNodePosition { start: 49, end: 67 }
      )
    )
  }

  #[test]
  fn parse_unfinished_doc() {
    let doc = r#"
        <poml syntax="markdown">
            <p> Hello, {{ name }}! </p>
        "#;
    let mut parser = PomlParser::from_poml_str(doc);
    let node = parser.parse_as_node();
    assert!(node.is_err());
  }

  #[test]
  fn parse_close_tag_without_open() {
    let doc = r#"
        <poml syntax="markdown">
            <h1> Hello, {{ name }}! </p>
        </poml>
        "#;
    let mut parser = PomlParser::from_poml_str(doc);
    let node = parser.parse_as_node();
    assert!(node.is_err());
    let err = node.unwrap_err();
    assert!(err.message.contains("position (2, 37)"))
  }

  #[test]
  fn parse_no_poml_root_doc() {
    let doc = r#"
            <p> Hello, {{ name }}! </p>
            <p> Good bye, {{ name }}! </p>
        "#;
    let mut parser = PomlParser::from_poml_str(doc);
    let node = parser.parse_as_node().unwrap();
    assert_eq!(node.children.iter().filter(|v| v.is_tag()).count(), 2);
  }

  #[test]
  fn parse_multiple_same_key_attribute_doc() {
    let doc = r#"
        <poml syntax="markdown" syntax="json"></poml>
        "#;
    let mut parser = PomlParser::from_poml_str(doc);
    assert!(parser.parse_as_node().is_err());
  }
}
