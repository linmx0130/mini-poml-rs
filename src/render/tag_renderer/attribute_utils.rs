/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

#[derive(Debug, PartialEq)]
pub enum CaptionStyle {
  Hidden,
  Bold,
  Header,
  Plain,
}

/**
 * Get caption style value from the attributes. If the value is not provided
 * or invalid, the default one will be used.
 */
pub fn get_caption_style(
  attribute_values: &[(String, String)],
  default: CaptionStyle,
) -> CaptionStyle {
  let caption_style_str = attribute_values
    .iter()
    .find(|v| v.0 == "captionStyle")
    .map(|(_, value)| value.as_str());

  match caption_style_str {
    Some("hidden") => CaptionStyle::Hidden,
    Some("bold") => CaptionStyle::Bold,
    Some("header") => CaptionStyle::Header,
    Some("plain") => CaptionStyle::Plain,
    _ => default,
  }
}
