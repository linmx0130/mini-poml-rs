pub mod error;
pub mod parser;
pub mod render;

pub use parser::PomlParser;

#[derive(Debug, PartialEq)]
pub enum PomlNode<'a> {
  Tag(PomlTagNode<'a>),
  Text(&'a str),
}

#[derive(Debug, PartialEq)]
pub struct PomlTagNode<'a> {
  pub name: &'a str,
  pub attributes: Vec<(&'a str, &'a str)>,
  pub children: Vec<PomlNode<'a>>,
}
