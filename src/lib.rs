pub mod error;
pub mod parser;

#[derive(Debug)]
pub enum PomlNode {
  Tag(PomlTagNode),
  Text(String),
}

#[derive(Debug)]
pub struct PomlTagNode {
  pub name: String,
  pub attributes: Vec<(String, String)>,
  pub children: Vec<PomlNode>,
}

#[derive(Debug, PartialEq)]
pub enum PomlElementKind {
  Tag,
  Text,
}

#[derive(Debug, PartialEq)]
pub struct PomlElement {
  pub kind: PomlElementKind,
  pub start_pos: usize,
  pub end_pos: usize,
}
