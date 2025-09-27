/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::PomlTagNode;
use crate::error::Result;

pub trait TagRenderer: Clone {
  fn render_tag(
    &self,
    tag: &PomlTagNode,
    attribute_values: &[(String, String)],
    children_result: Vec<String>,
    source_buf: &[u8],
  ) -> Result<String>;
}

mod markdown;
pub use markdown::MarkdownTagRenderer;
pub(crate) mod attribute_utils;
