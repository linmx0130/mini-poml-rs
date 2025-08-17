pub fn is_false_value(value: &str) -> bool {
  let val = value.trim();
  match val {
    "0" | "false" | "" | "null" | "NaN" => true,
    _ => false,
  }
}

pub fn buf_match_str(buf: &[u8], pos: usize, pattern: &str) -> bool {
  if pos + pattern.len() > buf.len() {
    return false;
  }
  let pattern_buf = pattern.as_bytes();
  for i in 0..pattern_buf.len() {
    if buf[pos + i] != pattern_buf[i] {
      return false;
    }
  }
  true
}
