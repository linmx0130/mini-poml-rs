use clap::Parser;
use mini_poml_rs::MarkdownPomlRenderer;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::io;

/// Demo program to use mini-poml-rs to render POML files.
#[derive(Parser, Debug)]
struct Args {
  /// POML filename to render
  poml_filename: String,
  /// Optional JSON file to supply the context. Only an object is allowed in the json file.
  context_json_filename: Option<String>,
}

fn main() -> io::Result<()> {
  let args = Args::parse();
  let poml_file = fs::read_to_string(&args.poml_filename)?;
  let mut renderer = match args.context_json_filename {
    Some(f) => {
      let context_json = fs::read_to_string(&f)?;
      let Ok(Value::Object(context_value)) = serde_json::from_str(&context_json) else {
        return Err(std::io::Error::other("Failed to parse context json!"));
      };
      MarkdownPomlRenderer::create_from_doc_and_variables(&poml_file, context_value)
    }
    None => MarkdownPomlRenderer::create_from_doc_and_variables(&poml_file, HashMap::new()),
  };

  let output = renderer.render().unwrap();
  println!("{output}");
  Ok(())
}
