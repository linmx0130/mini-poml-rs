/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

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
  let parser = PomlParser::from_str(doc);
  let mut renderer = Renderer {
    parser: parser,
    context: context,
    tag_renderer: TestTagRenderer {},
  };

  let output = renderer.render().unwrap();
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
  let parser = PomlParser::from_str(doc);
  let mut renderer = Renderer {
    parser: parser,
    context: context,
    tag_renderer: MarkdownTagRenderer {},
  };

  let output = renderer.render().unwrap();
  assert_eq!(
    output.trim(),
    "**This** is an *important* up*date*.\n\nGuess what?"
  );
}

#[test]
fn test_let_tag() {
  let doc = r#"
            <poml syntax="markdown">
              <let name="name" value="world" />
              <p> Hello, {{name}}! </p>
            </poml>
        "#;
  let context = render_context::RenderContext::from_iter(HashMap::<String, Value>::new());
  let parser = PomlParser::from_str(doc);
  let mut renderer = Renderer {
    parser: parser,
    context: context,
    tag_renderer: TestTagRenderer {},
  };

  let output = renderer.render().unwrap();
  assert!(output.contains("Hello, world!"));
}

#[test]
fn test_if_attributes() {
  let doc = r#"
            <poml syntax="markdown">
              <let name="name" value="world" />
              <let name="isFrenchVisible" value="1" />
              <p if="{{isVisible}}"> Hello, {{name}}! </p>
              <p if="{{isFrenchVisible}}"> Bonjour, {{name}}! </p>
            </poml>
        "#;
  let context = render_context::RenderContext::from_iter(HashMap::<String, Value>::new());
  let parser = PomlParser::from_str(doc);
  let mut renderer = Renderer {
    parser: parser,
    context: context,
    tag_renderer: TestTagRenderer {},
  };

  let output = renderer.render().unwrap();
  assert!(!output.contains("Hello, world!"));
  assert!(output.contains("Bonjour, world!"));
}

#[test]
fn test_code_tag() {
  use crate::MarkdownPomlRenderer;
  let code_piece = r#"
import numpy as np

if __name__ == "__main__":
    x = np.random((3, 4))
    print(f"{x}")
"#
  .trim();
  let doc = format!(
    r#"
<poml syntax="markdown">
  <let name="name" value="world" />
<code lang="python">
{}
</code>
</poml>
"#,
    code_piece
  );
  let mut renderer = MarkdownPomlRenderer::create_from_doc_and_variables(&doc, HashMap::new());
  let output = renderer.render().unwrap();
  assert!(output.contains("```python"));
  assert!(output.contains(code_piece))
}

#[test]
fn test_intentional_blocks() {
  use crate::MarkdownPomlRenderer;
  let doc = r#"
<poml syntax="markdown">
  <role>You're a helpful assistant.</role>
  <task>
  Answer questions as much as possible. 
  
  <stepwise-instructions>
    Work! Work! Work!
  </stepwise-instructions>
  </task>
</poml>
"#;
  let mut renderer = MarkdownPomlRenderer::create_from_doc_and_variables(&doc, HashMap::new());
  let output = renderer.render().unwrap();
  assert!(output.contains("# Role\n\nYou're a helpful assistant."));
  assert!(output.contains("# Task\n\n"));
  assert!(output.contains("## Stepwise Instructions\n\n"));
}
