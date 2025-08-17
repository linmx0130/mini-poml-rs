use super::*;
use crate::PomlParser;
use serde_json::json;
use std::collections::HashMap;
use tag_renderer::{MarkdownTagRenderer, TestTagRenderer};

#[test]
fn test_render_content() {
  let doc = r#"
            <poml syntax="markdown">
                <p> Hello, {{name}}! </p>
            </poml>
        "#;
  let mut variables = HashMap::new();
  variables.insert("name".to_owned(), json!("world"));
  let context = render_context::RenderContext::from_iter(variables);
  let mut renderer = Renderer {
    context: context,
    tag_renderer: TestTagRenderer {},
  };

  let mut parser = PomlParser::from_str(doc);
  let node = parser.parse_as_node().unwrap();
  let output = renderer.render(&PomlNode::Tag(node)).unwrap();
  assert!(output.contains("Hello, world!"));
}

#[test]
fn test_render_markdown() {
  let doc = r#"
            <poml syntax="markdown">
                <p> <b>This</b> is an <i>important</i> up<i>date</i>.</p>
                <p>Guess what?</p>
            </poml>
        "#;
  let variables: HashMap<String, Value> = HashMap::new();
  let context = render_context::RenderContext::from_iter(variables);
  let mut renderer = Renderer {
    context: context,
    tag_renderer: MarkdownTagRenderer {},
  };

  let mut parser = PomlParser::from_str(doc);
  let node = parser.parse_as_node().unwrap();
  let output = renderer.render(&PomlNode::Tag(node)).unwrap();
  assert_eq!(
    output.trim(),
    "**This** is an *important* up*date*.\n\nGuess what?"
  );
}
