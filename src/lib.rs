pub mod error;
pub mod parser;
pub mod render;

pub use parser::PomlParser;

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
}
