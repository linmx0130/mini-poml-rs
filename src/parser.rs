use crate::error::{Error, ErrorKind, Result};
use crate::{PomlElement, PomlElementKind};

#[derive(Debug)]
pub struct PomlParser<'a> {
  pub buf: &'a [u8],
  pub pos: usize,
}

impl<'a> PomlParser<'a> {
  pub fn from_str(s: &'a str) -> PomlParser<'a> {
    PomlParser {
      buf: s.as_bytes(),
      pos: 0,
    }
  }

  pub fn next_element(&mut self) -> Result<Option<PomlElement>> {
    while self.pos < self.buf.len() {
      let c = char::from(self.buf[self.pos]);
      match c {
        c if c.is_ascii_whitespace() => {
          self.pos += 1;
        }
        '<' => {
          let start_pos = self.pos;
          let end_pos = match self.seek_gt_char(start_pos + 1) {
            Some(p) => p,
            None => {
              return Err(Error {
                kind: ErrorKind::ParserError,
                message: Some(format!("Tag starting from {} is not complete", start_pos)),
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
    while pos < self.buf.len() {
      match self.buf[pos] {
        b'>' => {
          return Some(pos + 1);
        }
        _ => {
          pos = pos + 1;
        }
      }
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
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::*;
  #[test]
  fn tokenize_simple_poml() {
    let doc = r#"
        <poml syntax="markdown">
            <p> Hello, {{ name }}! </p>
        </poml>
        "#;
    let mut parser = PomlParser::from_str(doc);
    let mut elements = Vec::new();
    loop {
      match parser.next_element().unwrap() {
        Some(e) => elements.push(e),
        None => break,
      }
    }
    assert_eq!(elements.len(), 5);
    assert_eq!(elements[0].kind, PomlElementKind::Tag);
    assert_eq!(
      &doc[elements[0].start_pos..elements[0].end_pos],
      "<poml syntax=\"markdown\">"
    );
    assert_eq!(elements[1].kind, PomlElementKind::Tag);
    assert_eq!(&doc[elements[1].start_pos..elements[1].end_pos], "<p>");
    assert_eq!(elements[2].kind, PomlElementKind::Text);
    assert_eq!(
      &doc[elements[2].start_pos..elements[2].end_pos],
      "Hello, {{ name }}! "
    );
    assert_eq!(elements[3].kind, PomlElementKind::Tag);
    assert_eq!(&doc[elements[3].start_pos..elements[3].end_pos], "</p>");
    assert_eq!(elements[4].kind, PomlElementKind::Tag);
    assert_eq!(&doc[elements[4].start_pos..elements[4].end_pos], "</poml>");
  }
}
