/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum CaptionStyle {
  Hidden,
  Bold,
  Header,
  Plain,
}

/**
 * Get caption style value and caption colon setting from the attributes.
 * If the caption style value is not provided or invalid, the default one will be used.
 */
pub fn get_caption_style_and_colon(
  attribute_values: &[(String, String)],
  default_style: CaptionStyle,
) -> (CaptionStyle, bool) {
  let style = get_caption_style(attribute_values, default_style);
  let colon = get_caption_colon_value(attribute_values, style);
  (style, colon)
}

/**
 * Get caption style value from the attributes. If the value is not provided
 * or invalid, the default one will be used.
 */
fn get_caption_style(attribute_values: &[(String, String)], default: CaptionStyle) -> CaptionStyle {
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

/**
 * Get caption colon value from the attributes.
 *
 * By default, it will be true if caption style is plain or bold, and false for
 * other caption styles.
 */
fn get_caption_colon_value(
  attribute_values: &[(String, String)],
  caption_style: CaptionStyle,
) -> bool {
  use crate::render::utils::is_false_value;
  let caption_colon_str = attribute_values
    .iter()
    .find(|v| v.0 == "captionColon")
    .map(|(_, value)| value.as_str());
  match caption_colon_str {
    Some(v) => !is_false_value(v),
    None => matches!(caption_style, CaptionStyle::Plain | CaptionStyle::Bold),
  }
}
