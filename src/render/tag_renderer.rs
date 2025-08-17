pub trait TagRenderer {
  fn render_tag(
    &self,
    name: &str,
    attribute_values: &Vec<(String, String)>,
    children_result: Vec<String>,
  ) -> String;
}

/**
 * The tag render that renders nothing except dumping the
 * name, attributes and children it receives.
 */
pub struct TestTagRenderer {}

impl TagRenderer for TestTagRenderer {
  fn render_tag(
    &self,
    name: &str,
    attribute_values: &Vec<(String, String)>,
    children_result: Vec<String>,
  ) -> String {
    let mut answer = String::new();
    answer += &format!("Name: {}\n", name);
    for (key, value) in attribute_values {
      answer += &format!("  - {}: {}\n", key, value);
    }
    answer += "=====\n";
    for c in children_result {
      answer += &format!("{}\n", c);
    }
    answer += "=====\n";
    answer
  }
}
