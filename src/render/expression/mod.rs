/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub(crate) mod evaluate;
pub(crate) mod tokenize;
pub(crate) mod utils;
use super::render_context::RenderContext;
use crate::error::Result;
use serde_json::Value;

pub fn evaluate_expression(expression: &str, context: &RenderContext) -> Result<Value> {
  let tokens = tokenize::tokenize_expression(expression.as_bytes())?;
  evaluate::evaluate_expression_tokens(&tokens, context)
}
