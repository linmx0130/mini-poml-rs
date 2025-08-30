/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
use super::*;
use serde_json::json;

#[test]
fn test_evaluate_reference() {
  let Value::Object(variables) = json!({
      "my": {"home":"127.0.0.1"},
      "count": 5
  }) else {
    panic!();
  };
  let context = RenderContext::from(variables);
  assert_eq!(
    recognize_next_value(
      &[
        ExpressionToken::Ref(b"my"),
        ExpressionToken::Dot,
        ExpressionToken::Ref(b"home"),
      ],
      0,
      &context
    )
    .unwrap()
    .0,
    json!("127.0.0.1")
  );
  assert_eq!(
    evaluate_reference("count".as_bytes(), &context).unwrap(),
    json!(5)
  );
  assert_eq!(
    recognize_next_value(
      &[
        ExpressionToken::Ref(b"my"),
        ExpressionToken::Dot,
        ExpressionToken::Ref(b"car")
      ],
      0,
      &context
    )
    .unwrap()
    .0,
    Value::Null
  );
  assert!(
    recognize_next_value(
      &[
        ExpressionToken::Ref(b"my"),
        ExpressionToken::Dot,
        ExpressionToken::Ref(b"car"),
        ExpressionToken::Dot,
        ExpressionToken::Ref(b"window")
      ],
      0,
      &context
    )
    .is_err()
  );
}

#[test]
fn test_evaluate_numbers() {
  assert_eq!(evaluate_number("123".as_bytes()).unwrap(), json!(123));
  assert_eq!(evaluate_number("0.55".as_bytes()).unwrap(), json!(0.55));
}

#[test]
fn test_evaluate_array() {
  let Value::Object(variables) = json!({}) else {
    panic!();
  };
  let context = RenderContext::from(variables);
  assert_eq!(
    evaluate_expression_tokens(
      &[
        ExpressionToken::LeftBracket,
        ExpressionToken::Number(b"1"),
        ExpressionToken::Comma,
        ExpressionToken::String(b"\'2\'"),
        ExpressionToken::Comma,
        ExpressionToken::LeftBracket,
        ExpressionToken::Ref(b"true"),
        ExpressionToken::Comma,
        ExpressionToken::Ref(b"false"),
        ExpressionToken::Comma,
        ExpressionToken::RightBracket,
        ExpressionToken::Comma,
        ExpressionToken::RightBracket,
      ],
      &context
    )
    .unwrap(),
    json!([1, "2", [true, false]])
  );
}

#[test]
fn test_evaluate_not() {
  let Value::Object(variables) = json!({"flag": false}) else {
    panic!();
  };
  let context = RenderContext::from(variables);
  assert_eq!(
    evaluate_expression_tokens(
      &[ExpressionToken::Exclamation, ExpressionToken::Ref(b"flag"),],
      &context
    )
    .unwrap(),
    json!(true)
  );
}

#[test]
fn test_evaluate_plus_arith() {
  let Value::Object(variables) = json!({
      "a": 1,
      "b": 2,
      "s": "a"
  }) else {
    panic!();
  };
  let context = RenderContext::from(variables);
  let (result, pos) = evaluate_expression_value(
    &[
      ExpressionToken::Ref(b"a"),
      ExpressionToken::ArithOp(b"+"),
      ExpressionToken::Ref(b"b"),
    ],
    0,
    &context,
  )
  .unwrap();
  assert_eq!(result, json!(3));
  assert_eq!(pos, 3);

  let (result, pos) = evaluate_expression_value(
    &[
      ExpressionToken::Ref(b"a"),
      ExpressionToken::ArithOp(b"+"),
      ExpressionToken::Ref(b"s"),
    ],
    0,
    &context,
  )
  .unwrap();
  assert_eq!(result, json!("1a"));
  assert_eq!(pos, 3);
}

#[test]
fn test_evaluate_minus_arith() {
  let Value::Object(variables) = json!({
      "a": 1,
      "b": 2,
      "c": 3.0,
  }) else {
    panic!();
  };
  let context = RenderContext::from(variables);
  let (result, pos) = evaluate_expression_value(
    &[
      ExpressionToken::Ref(b"a"),
      ExpressionToken::ArithOp(b"-"),
      ExpressionToken::Ref(b"b"),
    ],
    0,
    &context,
  )
  .unwrap();
  assert_eq!(result, json!(-1));
  assert_eq!(pos, 3);

  let (result, pos) = evaluate_expression_value(
    &[
      ExpressionToken::Ref(b"a"),
      ExpressionToken::ArithOp(b"-"),
      ExpressionToken::Ref(b"c"),
    ],
    0,
    &context,
  )
  .unwrap();
  assert_eq!(result, json!(-2.0));
  assert_eq!(pos, 3);
}

#[test]
fn test_evaluate_parenthesis() {
  let Value::Object(variables) = json!({
      "a": 1,
      "b": 2,
      "s": " "
  }) else {
    panic!();
  };
  let context = RenderContext::from(variables);
  let (result, pos) = evaluate_expression_value(
    &[
      ExpressionToken::Ref(b"a"),
      ExpressionToken::ArithOp(b"-"),
      ExpressionToken::LeftParenthesis,
      ExpressionToken::Ref(b"b"),
      ExpressionToken::ArithOp(b"-"),
      ExpressionToken::Ref(b"a"),
      ExpressionToken::RightParenthesis,
    ],
    0,
    &context,
  )
  .unwrap();
  assert_eq!(result, json!(0));
  assert_eq!(pos, 7);
}

#[test]
fn test_mix_int_string_plus() {
  let Value::Object(variables) = json!({
      "a": 1,
      "b": 2,
      "s": " "
  }) else {
    panic!();
  };
  let context = RenderContext::from(variables);
  let (result, pos) = evaluate_expression_value(
    &[
      ExpressionToken::Ref(b"s"),
      ExpressionToken::ArithOp(b"+"),
      ExpressionToken::LeftParenthesis,
      ExpressionToken::Ref(b"a"),
      ExpressionToken::ArithOp(b"+"),
      ExpressionToken::Ref(b"b"),
      ExpressionToken::RightParenthesis,
    ],
    0,
    &context,
  )
  .unwrap();
  assert_eq!(result, json!(" 3"));
  assert_eq!(pos, 7);

  let (result, pos) = evaluate_expression_value(
    &[
      ExpressionToken::Ref(b"s"),
      ExpressionToken::ArithOp(b"+"),
      ExpressionToken::Ref(b"a"),
      ExpressionToken::ArithOp(b"+"),
      ExpressionToken::Ref(b"b"),
    ],
    0,
    &context,
  )
  .unwrap();
  assert_eq!(result, json!(" 12"));
  assert_eq!(pos, 5);
}

#[test]
fn test_evaluate_times() {
  let Value::Object(variables) = json!({
      "a": 1,
      "b": 2,
      "c": 4,
  }) else {
    panic!();
  };
  let context = RenderContext::from(variables);
  let (result, pos) = evaluate_expression_value(
    &[
      ExpressionToken::Ref(b"a"),
      ExpressionToken::ArithOp(b"+"),
      ExpressionToken::Ref(b"b"),
      ExpressionToken::ArithOp(b"*"),
      ExpressionToken::Ref(b"c"),
    ],
    0,
    &context,
  )
  .unwrap();
  assert_eq!(result, json!(9));
  assert_eq!(pos, 5);
}

#[test]
fn test_evaluate_times_and_divide() {
  let Value::Object(variables) = json!({
      "a": 1,
      "b": 2,
      "c": 4,
  }) else {
    panic!();
  };
  let context = RenderContext::from(variables);
  let (result, pos) = evaluate_expression_value(
    &[
      ExpressionToken::LeftParenthesis,
      ExpressionToken::Ref(b"a"),
      ExpressionToken::ArithOp(b"+"),
      ExpressionToken::Ref(b"b"),
      ExpressionToken::ArithOp(b"*"),
      ExpressionToken::Ref(b"c"),
      ExpressionToken::RightParenthesis,
      ExpressionToken::ArithOp(b"/"),
      ExpressionToken::Ref(b"b"),
    ],
    0,
    &context,
  )
  .unwrap();
  assert_eq!(result, json!(4.5));
  assert_eq!(pos, 9);
}

#[test]
fn test_strict_equal_operator() {
  let Value::Object(variables) = json!({
      "a": 1,
      "b": 1,
      "c": "1",
  }) else {
    panic!();
  };
  let context = RenderContext::from(variables);
  let (result, _) = evaluate_expression_value(
    &[
      ExpressionToken::Ref(b"a"),
      ExpressionToken::ArithOp(b"==="),
      ExpressionToken::Ref(b"b"),
    ],
    0,
    &context,
  )
  .unwrap();
  assert_eq!(result, json!(true));
  let (result, _) = evaluate_expression_value(
    &[
      ExpressionToken::Ref(b"a"),
      ExpressionToken::ArithOp(b"==="),
      ExpressionToken::Ref(b"c"),
    ],
    0,
    &context,
  )
  .unwrap();
  assert_eq!(result, json!(false));
}

#[test]
fn test_logical_operator() {
  let Value::Object(variables) = json!({
      "a": true,
      "b": false,
  }) else {
    panic!();
  };
  let context = RenderContext::from(variables);
  let (result, _) = evaluate_expression_value(
    &[
      ExpressionToken::Ref(b"a"),
      ExpressionToken::ArithOp(b"&&"),
      ExpressionToken::Ref(b"b"),
    ],
    0,
    &context,
  )
  .unwrap();
  assert_eq!(result, json!(false));
  let (result, _) = evaluate_expression_value(
    &[
      ExpressionToken::Ref(b"a"),
      ExpressionToken::ArithOp(b"||"),
      ExpressionToken::Ref(b"c"),
    ],
    0,
    &context,
  )
  .unwrap();
  assert_eq!(result, json!(true));

  let (result, _) = evaluate_expression_value(
    &[
      ExpressionToken::Ref(b"b"),
      ExpressionToken::ArithOp(b"||"),
      ExpressionToken::ArithOp(b"!"),
      ExpressionToken::Ref(b"a"),
    ],
    0,
    &context,
  )
  .unwrap();
  assert_eq!(result, json!(false));

  let (result, _) = evaluate_expression_value(
    &[
      ExpressionToken::ArithOp(b"!"),
      ExpressionToken::LeftParenthesis,
      ExpressionToken::Ref(b"a"),
      ExpressionToken::ArithOp(b"&&"),
      ExpressionToken::Ref(b"b"),
      ExpressionToken::RightParenthesis,
    ],
    0,
    &context,
  )
  .unwrap();
  assert_eq!(result, json!(true));
}
