//! Unit tests for the `char` module's native ops.
//!
//! Covers Unicode category/case atoms, char-indexed access, bounds errors,
//! code-point conversion (incl. invalid code points), and the `chars`/
//! `from_chars` round-trip.

use serde_json::json;

use handl_ops::{OpContext, OpError};

use crate::ops::char_op_registry;

/// The ops are pure — a context that rejects any capability use.
struct NullCtx;

impl OpContext for NullCtx {
    fn eval_expr(
        &mut self,
        _expr: &str,
        _vars: &[(String, serde_json::Value)],
    ) -> Result<serde_json::Value, OpError> {
        Err(OpError::Other("no eval in tests".into()))
    }

    fn try_call_value(
        &mut self,
        _callee: &serde_json::Value,
        _args: &[serde_json::Value],
    ) -> Option<Result<serde_json::Value, OpError>> {
        None
    }
}

fn registry() -> std::sync::Arc<handl_ops::OpRegistry> {
    std::sync::Arc::new(char_op_registry())
}

fn call(name: &str, args: &[serde_json::Value]) -> serde_json::Value {
    let op = registry()
        .get(&format!("char.{name}"))
        .unwrap_or_else(|| panic!("char.{name} must exist"));
    op.call(args, &mut NullCtx)
        .unwrap_or_else(|e| panic!("char.{name}({args:?}) failed: {e}"))
}

fn call_err(name: &str, args: &[serde_json::Value]) -> String {
    let op = registry()
        .get(&format!("char.{name}"))
        .unwrap_or_else(|| panic!("char.{name} must exist"));
    op.call(args, &mut NullCtx).unwrap_err().to_string()
}

#[test]
fn registry_covers_all_declared_ops() {
    let reg = char_op_registry();
    for name in crate::CHAR_OPS {
        assert!(reg.contains(name), "missing op {name}");
    }
    assert_eq!(reg.len(), crate::CHAR_OPS.len());
}

#[test]
fn category_atoms() {
    assert_eq!(call("is_whitespace", &[json!(" ")]), json!(true));
    assert_eq!(call("is_whitespace", &[json!("a")]), json!(false));
    // Unicode whitespace (non-breaking space, em space).
    assert_eq!(call("is_whitespace", &[json!("\u{00A0}")]), json!(true));
    assert_eq!(call("is_whitespace", &[json!("\u{2003}")]), json!(true));

    assert_eq!(call("is_alpha", &[json!("a")]), json!(true));
    assert_eq!(call("is_alpha", &[json!("é")]), json!(true));
    assert_eq!(call("is_alpha", &[json!("9")]), json!(false));

    assert_eq!(call("is_numeric", &[json!("9")]), json!(true));
    assert_eq!(call("is_numeric", &[json!("a")]), json!(false));
    // Unicode numeric (superscript two).
    assert_eq!(call("is_numeric", &[json!("²")]), json!(true));

    assert_eq!(call("is_alnum", &[json!("a")]), json!(true));
    assert_eq!(call("is_alnum", &[json!("9")]), json!(true));
    assert_eq!(call("is_alnum", &[json!(" ")]), json!(false));

    assert_eq!(call("is_uppercase", &[json!("A")]), json!(true));
    assert_eq!(call("is_uppercase", &[json!("a")]), json!(false));
    assert_eq!(call("is_lowercase", &[json!("a")]), json!(true));
    assert_eq!(call("is_lowercase", &[json!("A")]), json!(false));
}

#[test]
fn case_mapping() {
    assert_eq!(call("to_upper", &[json!("a")]), json!("A"));
    assert_eq!(call("to_lower", &[json!("A")]), json!("a"));
    assert_eq!(call("to_title", &[json!("a")]), json!("A"));
    // Expansion: ß → SS.
    assert_eq!(call("to_upper", &[json!("ß")]), json!("SS"));
    assert_eq!(call("to_lower", &[json!("É")]), json!("é"));
}

#[test]
fn code_point_round_trip() {
    assert_eq!(call("code_point", &[json!("a")]), json!(97.0));
    assert_eq!(call("code_point", &[json!("é")]), json!(233.0));
    assert_eq!(call("from_code_point", &[json!(97.0)]), json!("a"));
    assert_eq!(call("from_code_point", &[json!(233.0)]), json!("é"));

    // Invalid code points error.
    assert!(call_err("from_code_point", &[json!(-1.0)]).contains("expected"));
    assert!(call_err("from_code_point", &[json!(0xD800 as f64)]).contains("invalid code point"));
    assert!(call_err("from_code_point", &[json!(0x110000 as f64)]).contains("invalid code point"));

    // code_point requires a single char.
    assert!(call_err("code_point", &[json!("ab")]).contains("single char"));
}

#[test]
fn char_indexed_access() {
    // Indices are char indices, not bytes.
    assert_eq!(call("at", &[json!("héllo"), json!(1.0)]), json!("é"));
    assert_eq!(call("at", &[json!("abc"), json!(0.0)]), json!("a"));
    assert_eq!(call("at", &[json!("abc"), json!(2.0)]), json!("c"));
    assert!(call_err("at", &[json!("abc"), json!(3.0)]).contains("out of bounds"));

    assert_eq!(
        call("slice", &[json!("héllo"), json!(1.0), json!(3.0)]),
        json!("él")
    );
    assert_eq!(
        call("slice", &[json!("abc"), json!(0.0), json!(0.0)]),
        json!("")
    );
    // Out-of-range end clamps to char count.
    assert_eq!(
        call("slice", &[json!("abc"), json!(1.0), json!(99.0)]),
        json!("bc")
    );

    assert_eq!(call("length", &[json!("héllo")]), json!(5.0));
    assert_eq!(call("length", &[json!("")]), json!(0.0));

    assert_eq!(call("concat", &[json!("ab"), json!("çd")]), json!("abçd"));

    assert_eq!(
        call("code_point_at", &[json!("héllo"), json!(1.0)]),
        json!(233.0)
    );
    assert!(call_err("code_point_at", &[json!("abc"), json!(3.0)]).contains("out of bounds"));
}

#[test]
fn chars_and_from_chars_round_trip() {
    assert_eq!(
        call("chars", &[json!("héllo")]),
        json!(["h", "é", "l", "l", "o"])
    );
    assert_eq!(call("chars", &[json!("")]), json!([]));
    assert_eq!(call("from_chars", &[json!(["a", "b", "ç"])]), json!("abç"));
    assert_eq!(call("from_chars", &[json!([])]), json!(""));

    // Round-trip: from_chars(chars(s)) == s.
    let s = "héllo wörld — ß";
    let chars = call("chars", &[json!(s)]);
    assert_eq!(call("from_chars", &[chars]), json!(s));
}
