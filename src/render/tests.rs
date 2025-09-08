/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use super::*;
use crate::{PomlParser, PomlTagNode};
use serde_json::json;
use std::collections::HashMap;
use tag_renderer::{MarkdownTagRenderer, TagRenderer};

/**
 * The tag render that renders nothing except dumping the
 * name, attributes and children it receives.
 */
#[derive(Clone)]
struct TestTagRenderer {}

impl TagRenderer for TestTagRenderer {
  fn render_tag(
    &self,
    tag: &PomlTagNode,
    attribute_values: &[(String, String)],
    children_result: Vec<String>,
    _source_buf: &[u8],
  ) -> Result<String> {
    let mut answer = String::new();
    answer += &format!("Name: {}\n", tag.name);
    for (key, value) in attribute_values {
      answer += &format!("  - {key}: {value}\n");
    }
    answer += "=====\n";
    for c in children_result {
      answer += &format!("{c}\n");
    }
    answer += "=====\n";
    Ok(answer)
  }
}

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
    parser,
    context,
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
    parser,
    context,
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
    parser,
    context,
    tag_renderer: TestTagRenderer {},
  };

  let output = renderer.render().unwrap();
  assert!(output.contains("Hello, world!"));
}

#[test]
fn test_let_tag_children_value() {
  let doc = r#"
            <poml syntax="markdown">
              <let name="name">world</let>
              <p> Hello, {{name}}! </p>
            </poml>
        "#;
  let context = render_context::RenderContext::from_iter(HashMap::<String, Value>::new());
  let parser = PomlParser::from_str(doc);
  let mut renderer = Renderer {
    parser,
    context,
    tag_renderer: TestTagRenderer {},
  };

  let output = renderer.render().unwrap();
  assert!(output.contains("Hello, world!"));
}

#[test]
fn test_let_tag_duplicated_values() {
  let doc = r#"
            <poml syntax="markdown">
              <let name="name" value="le monde">world</let>
              <p> Hello, {{name}}! </p>
            </poml>
        "#;
  let context = render_context::RenderContext::from_iter(HashMap::<String, Value>::new());
  let parser = PomlParser::from_str(doc);
  let mut renderer = Renderer {
    parser,
    context,
    tag_renderer: TestTagRenderer {},
  };

  assert!(renderer.render().is_err());
}
#[test]
fn test_let_tag_with_type() {
  let doc = r#"
            <poml syntax="markdown">
              <let name="count" value="3" type="number" />
              <p> Count: {{count}} </p>
              <let name="isVisible" value="0" type="boolean" />
              <p if="{{ isVisible }}"> Not visible </p>
            </poml>
        "#;
  let context = render_context::RenderContext::from_iter(HashMap::<String, Value>::new());
  let parser = PomlParser::from_str(doc);
  let mut renderer = Renderer {
    parser,
    context,
    tag_renderer: TestTagRenderer {},
  };

  let output = renderer.render().unwrap();
  assert!(output.contains("Count: 3"));
  assert!(!output.contains("Not visible"));
}

#[test]
fn test_let_tag_with_type_with_invalid_value() {
  let doc = r#"
            <poml syntax="markdown">
              <let name="count" value="three" type="integer" />
              <p> Count: {{count}} </p>
            </poml>
        "#;
  let context = render_context::RenderContext::from_iter(HashMap::<String, Value>::new());
  let parser = PomlParser::from_str(doc);
  let mut renderer = Renderer {
    parser,
    context,
    tag_renderer: TestTagRenderer {},
  };

  let output = renderer.render();
  assert!(output.is_err());
}

#[test]
fn test_if_attributes() {
  let doc = r#"
            <poml syntax="markdown">
              <let name="name" value="world" />
              <let name="isFrenchVisible" value="1" />
              <p if="{{isVisible}}"> Hello, {{name}}! </p>
              <p if="{{isFrenchVisible}}"> Bonjour, {{name}}! </p>
              <p if="{{ !isVisible }}"> Something is hidden! </p>
            </poml>
        "#;
  let context = render_context::RenderContext::from_iter(HashMap::<String, Value>::new());
  let parser = PomlParser::from_str(doc);
  let mut renderer = Renderer {
    parser,
    context,
    tag_renderer: TestTagRenderer {},
  };

  let output = renderer.render().unwrap();
  assert!(!output.contains("Hello, world!"));
  assert!(output.contains("Bonjour, world!"));
  assert!(output.contains("Something is hidden"));
}

#[test]
fn test_for_attributes() {
  let doc = r#"
            <poml syntax="markdown">
              <p for="name in ['apple', 'banana', 'cherry']"> 
                Hello, {{name}}! {{ loop.index }}
                <p if="{{ loop.last }}"> End: {{ name }}</p>
              </p>
            </poml>
        "#;
  let context = render_context::RenderContext::from_iter(HashMap::<String, Value>::new());
  let parser = PomlParser::from_str(doc);
  let mut renderer = Renderer {
    parser,
    context,
    tag_renderer: TestTagRenderer {},
  };

  let output = renderer.render().unwrap();
  assert!(output.contains("Hello, apple! 0"));
  assert!(output.contains("Hello, banana! 1"));
  assert!(output.contains("Hello, cherry! 2"));
  assert!(output.contains("End: cherry"));
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
{code_piece}
</code>
</poml>
"#
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
  let mut renderer = MarkdownPomlRenderer::create_from_doc_and_variables(doc, HashMap::new());
  let output = renderer.render().unwrap();
  assert!(output.contains("# Role\n\nYou're a helpful assistant."));
  assert!(output.contains("# Task\n\n"));
  assert!(output.contains("## Stepwise Instructions\n\n"));
}

#[test]
fn test_variable_scope() {
  use crate::MarkdownPomlRenderer;
  let doc = r#"
<poml syntax="markdown">
  Start {{a}}
  <let name="a" value="a" />
  <p>
  <let name="a" value="b" />
  Mid {{a}}
  </p>
  End {{a}}
</poml>
"#;
  let mut variables = HashMap::new();
  variables.insert("a".to_owned(), json!("0"));
  let mut renderer = MarkdownPomlRenderer::create_from_doc_and_variables(doc, variables);
  let output = renderer.render().unwrap();
  assert!(output.contains("Start 0"));
  assert!(output.contains("Mid b"));
  assert!(output.contains("End a"));
}

#[test]
fn test_escaping() {
  use crate::MarkdownPomlRenderer;
  let doc = r#"
<poml syntax="markdown">
  Start #lt; #gt; ###
</poml>
"#;
  let mut renderer = MarkdownPomlRenderer::create_from_doc_and_variables(doc, HashMap::new());
  let output = renderer.render().unwrap();
  assert!(output.contains("Start < > ###"));
}

#[test]
fn test_header_and_section() {
  use crate::MarkdownPomlRenderer;
  let doc = r#"
<poml syntax="markdown">
  <h>Header 1</h>
  <section>
    <h>Header 2</h>
  </section>
  <h>Header 3</h>
</poml>
"#;
  let mut renderer = MarkdownPomlRenderer::create_from_doc_and_variables(doc, HashMap::new());
  let output = renderer.render().unwrap();
  assert!(output.contains("# Header 1"));
  assert!(output.contains("## Header 2"));
  assert!(output.contains("# Header 3"));
}

#[test]
fn test_include() {
  use crate::MarkdownPomlRenderer;
  let doc = r#"
<poml syntax="markdown">
  <p>File a</p>
  <include src="a.poml"/>
</poml>
"#;
  let a_doc = r#"<h>AAA</h>"#;
  let mut renderer = MarkdownPomlRenderer::create_from_doc_and_variables(doc, HashMap::new());
  renderer
    .context
    .file_mapping
    .insert("a.poml".to_owned(), a_doc.to_owned());
  let output = renderer.render().unwrap();
  assert!(output.contains("# AAA"));
}

#[test]
fn test_list_render() {
  use crate::MarkdownPomlRenderer;
  let doc = r#"
<poml syntax="markdown">
  <list listStyle="star">
    <item>Head</item>
    <item for="v in [1,2,3]">{{ v }}</item>
  </list>
</poml>
"#;
  let mut renderer = MarkdownPomlRenderer::create_from_doc_and_variables(doc, HashMap::new());
  let output = renderer.render().unwrap();
  assert!(output.contains("* Head"));
  assert!(output.contains("* 1"));
  assert!(output.contains("* 2"));
  assert!(output.contains("* 3"));
}

#[test]
fn test_nested_list_render() {
  use crate::MarkdownPomlRenderer;
  let doc = r#"
<poml syntax="markdown">
  <list>
    <item for="v in [1,2,3]">
      <list listStyle = "decimal">
        <item for="u in ['a', 'b', 'c']">({{v}}, {{ u }})</item>
      </list>
    </item>
  </list>
</poml>
"#;
  let mut renderer = MarkdownPomlRenderer::create_from_doc_and_variables(doc, HashMap::new());
  let output = renderer.render().unwrap();
  assert!(output.contains("-  1. (1, a)\n\t2. (1, b)\n\t3. (1, c)"));
  assert!(output.contains("-  1. (2, a)\n\t2. (2, b)\n\t3. (2, c)"));
}

#[test]
fn test_captioned_paragraph() {
  use crate::MarkdownPomlRenderer;
  let doc = r#"
<cp caption="Constraints">
  <h>Sub-list</h>
  <list>
    <item>Do not exceed 1000 tokens.</item>
    <item>Please use simple words.</item>
  </list>
</cp>"#;
  let mut renderer = MarkdownPomlRenderer::create_from_doc_and_variables(doc, HashMap::new());
  let output = renderer.render().unwrap();
  assert!(output.contains("# Constraints\n\n"));
  assert!(output.contains("## Sub-list\n\n"));
  assert!(output.contains("\n - Do not exceed 1000 tokens."));
}

#[test]
fn test_let_src_include() {
  use crate::MarkdownPomlRenderer;
  let doc = r#"
<poml syntax="markdown">
  <let name="foo" src="foo.json" />
  <p>{{ foo.bar }} </p>
</poml>
"#;
  let foo_doc = r#"{"bar":"fubar"}"#;
  let mut renderer = MarkdownPomlRenderer::create_from_doc_and_variables(doc, HashMap::new());
  renderer
    .context
    .file_mapping
    .insert("foo.json".to_owned(), foo_doc.to_owned());
  let output = renderer.render().unwrap();
  assert!(output.contains("fubar"));
}

#[test]
fn test_let_object() {
  use crate::MarkdownPomlRenderer;
  let doc = r#"
<poml syntax="markdown">
  <let name="foo" type="object">
    {"bar":"fubar"}
  </let>
  <p>{{ foo.bar }} </p>
</poml>
"#;
  let mut renderer = MarkdownPomlRenderer::create_from_doc_and_variables(doc, HashMap::new());
  let output = renderer.render().unwrap();
  assert!(output.contains("fubar"));
}

#[test]
fn test_let_array() {
  use crate::MarkdownPomlRenderer;
  let doc = r#"
<poml syntax="markdown">
  <let name="foo" type="array">
    [1, "foobar", true]
  </let>
  <p for="v in foo"> {{ v }} </p>
</poml>
"#;
  let mut renderer = MarkdownPomlRenderer::create_from_doc_and_variables(doc, HashMap::new());
  let output = renderer.render().unwrap();
  assert!(output.contains("1"));
  assert!(output.contains("foobar"));
  assert!(output.contains("true"));
}

#[test]
fn test_examples() {
  use crate::MarkdownPomlRenderer;
  let doc = r#"
<poml syntax="markdown">
  <examples>
    <example>
      <input>What is the capital of France?</input>
      <output>Paris</output>
    </example>
    <example>
      <input>What is the capital of Germany?</input>
      <output>Berlin</output>
    </example>
  </examples>
  <hint>No need to add any explanation</hint>
</poml>
"#;
  let mut renderer = MarkdownPomlRenderer::create_from_doc_and_variables(doc, HashMap::new());
  let output = renderer.render().unwrap();
  assert!(output.contains("# Examples"));
  assert!(output.contains("Paris\n"));
  assert!(output.contains("**Hint:** No need to add any explanation"));
}
