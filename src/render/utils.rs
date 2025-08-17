pub fn is_false_value(value: &str) -> bool {
  let val = value.trim();
  match val {
    "0" | "false" | "" | "null" | "NaN" => true,
    _ => false,
  }
}
