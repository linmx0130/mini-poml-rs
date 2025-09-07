/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub fn is_false_value(value: &str) -> bool {
  let val = value.trim();
  matches!(val, "0" | "false" | "" | "null" | "NaN")
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
