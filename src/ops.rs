//! Kernel-ABI [`Op`] impls for the `char` module — the handls (handlers)
//! for the `op`/`handl` declarations in `char-native.hnl`.
//!
//! All 18 functions are FFI atoms over `std::char`: category predicates
//! (`is_whitespace`, `is_alphabetic`, `is_numeric`, `is_alphanumeric`,
//! `is_uppercase`, `is_lowercase`), case maps (`to_uppercase`,
//! `to_lowercase`, `to_titlecase` via `to_uppercase` — Rust has no
//! titlecase), scalar conversion (`from_u32`, `as u32`), and
//! char-sequence access (`chars().nth`, `chars().skip/take`, `count`,
//! `concat`) with *character* indices, never byte offsets.
//!
//! `chars`/`from_chars` are O(n) bulk split/join so `str-handl`
//! algorithms iterate char lists without quadratic `at` scans.
//!
//! The ops are registered on the kernel [`handl_ops::OpRegistry`] ABI via
//! [`char_op_registry`]; the host-side `char` module wrapper (`CharModule`)
//! lives in `handl-plugin-lib-handl`, which imports this registry
//! through `OpRegistry::import_kernel`.

use std::sync::Arc;

use handl_ops::{Op, OpContext, OpError, OpRegistry};

/// The qualified op names declared in `char-native.hnl` — the keys
/// [`char_op_registry`] registers under. The local name (the segment after
/// the last `.`) is what `ModuleLookup` resolves inside the `char` module.
pub const CHAR_OPS: &[&str] = &[
    "char.is_whitespace",
    "char.is_alpha",
    "char.is_numeric",
    "char.is_alnum",
    "char.is_uppercase",
    "char.is_lowercase",
    "char.to_upper",
    "char.to_lower",
    "char.to_title",
    "char.code_point",
    "char.from_code_point",
    "char.at",
    "char.slice",
    "char.length",
    "char.concat",
    "char.code_point_at",
    "char.chars",
    "char.from_chars",
];

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn require_string(arg: &serde_json::Value) -> Result<&str, OpError> {
    arg.as_str()
        .ok_or_else(|| OpError::TypeMismatch("expected string, got non-string".into()))
}

fn require_one_string(args: &[serde_json::Value]) -> Result<&str, OpError> {
    let arg = args
        .first()
        .ok_or_else(|| OpError::TypeMismatch("expected 1 args, got 0".into()))?;
    require_string(arg)
}

fn require_int(arg: &serde_json::Value, what: &str) -> Result<usize, OpError> {
    arg.as_f64()
        .filter(|n| *n >= 0.0 && n.fract() == 0.0)
        .map(|n| n as usize)
        .ok_or_else(|| OpError::TypeMismatch(format!("expected {what}, got non-integer")))
}

fn require_char_index(args: &[serde_json::Value], pos: usize) -> Result<usize, OpError> {
    let arg = args
        .get(pos)
        .ok_or_else(|| OpError::TypeMismatch(format!("expected {pos}+ args")))?;
    require_int(arg, "non-negative integer index")
}

fn s(value: impl Into<String>) -> serde_json::Value {
    serde_json::Value::from(value.into())
}

fn b(value: bool) -> serde_json::Value {
    serde_json::Value::from(value)
}

fn num(value: f64) -> serde_json::Value {
    serde_json::json!(value)
}

fn type_mismatch(expected: &str, got: &str) -> OpError {
    OpError::TypeMismatch(format!("expected {expected}, got {got}"))
}

/// The single char of a 1-char string, or an error for empty/multi-char.
fn require_one_char(args: &[serde_json::Value]) -> Result<char, OpError> {
    let input = require_one_string(args)?;
    let mut it = input.chars();
    match (it.next(), it.next()) {
        (Some(c), None) => Ok(c),
        _ => Err(OpError::Other(format!(
            "expected a single char, got {input:?}"
        ))),
    }
}

// ---------------------------------------------------------------------------
// Character classification atoms
// ---------------------------------------------------------------------------

macro_rules! char_pred_fn {
    ($name:ident, $pred:expr) => {
        #[derive(Clone, Copy)]
        pub struct $name;
        impl Op for $name {
            fn call(
                &self,
                args: &[serde_json::Value],
                _ctx: &mut dyn OpContext,
            ) -> Result<serde_json::Value, OpError> {
                let c = require_one_char(args)?;
                let pred: fn(char) -> bool = $pred;
                Ok(b(pred(c)))
            }
        }
    };
}

char_pred_fn!(IsWhitespace, |c: char| c.is_whitespace());
char_pred_fn!(IsAlpha, |c: char| c.is_alphabetic());
char_pred_fn!(IsNumeric, |c: char| c.is_numeric());
char_pred_fn!(IsAlnum, |c: char| c.is_alphanumeric());
char_pred_fn!(IsUppercase, |c: char| c.is_uppercase());
char_pred_fn!(IsLowercase, |c: char| c.is_lowercase());

// ---------------------------------------------------------------------------
// Case-mapping atoms (may expand: ß → SS)
// ---------------------------------------------------------------------------

macro_rules! char_map_fn {
    ($name:ident, $map:expr) => {
        #[derive(Clone, Copy)]
        pub struct $name;
        impl Op for $name {
            fn call(
                &self,
                args: &[serde_json::Value],
                _ctx: &mut dyn OpContext,
            ) -> Result<serde_json::Value, OpError> {
                let c = require_one_char(args)?;
                let map: fn(char) -> String = $map;
                Ok(s(map(c)))
            }
        }
    };
}

char_map_fn!(ToUpper, |c: char| c.to_uppercase().collect());
char_map_fn!(ToLower, |c: char| c.to_lowercase().collect());
// Rust std::char has no to_titlecase; fall back to uppercase (titlecase
// digraphs like ǲ require an ICU backend).
char_map_fn!(ToTitle, |c: char| c.to_uppercase().collect());

// ---------------------------------------------------------------------------
// Scalar-value conversion atoms
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
pub struct CodePoint;
impl Op for CodePoint {
    fn call(
        &self,
        args: &[serde_json::Value],
        _ctx: &mut dyn OpContext,
    ) -> Result<serde_json::Value, OpError> {
        let c = require_one_char(args)?;
        Ok(num(c as u32 as f64))
    }
}

#[derive(Clone, Copy)]
pub struct FromCodePoint;
impl Op for FromCodePoint {
    fn call(
        &self,
        args: &[serde_json::Value],
        _ctx: &mut dyn OpContext,
    ) -> Result<serde_json::Value, OpError> {
        let arg = args.first().ok_or_else(|| type_mismatch("1 args", "0"))?;
        let n = arg
            .as_f64()
            .filter(|n| *n >= 0.0 && n.fract() == 0.0)
            .ok_or_else(|| type_mismatch("non-negative integer", "non-integer"))?;
        let cp = n as u32;
        char::from_u32(cp)
            .map(|c| s(c.to_string()))
            .ok_or_else(|| OpError::Other(format!("invalid code point {cp}")))
    }
}

// ---------------------------------------------------------------------------
// Char-sequence access atoms (char indices, not byte offsets)
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
pub struct At;
impl Op for At {
    fn call(
        &self,
        args: &[serde_json::Value],
        _ctx: &mut dyn OpContext,
    ) -> Result<serde_json::Value, OpError> {
        let input = require_one_string(args)?;
        let index = require_char_index(args, 1)?;
        let char_count = input.chars().count();
        input
            .chars()
            .nth(index)
            .map(|c| s(c.to_string()))
            .ok_or_else(|| {
                OpError::Other(format!(
                    "index {index} out of bounds (char count: {char_count})"
                ))
            })
    }
}

#[derive(Clone, Copy)]
pub struct Slice;
impl Op for Slice {
    fn call(
        &self,
        args: &[serde_json::Value],
        _ctx: &mut dyn OpContext,
    ) -> Result<serde_json::Value, OpError> {
        let input = require_one_string(args)?;
        let start = require_char_index(args, 1)?;
        let end = require_char_index(args, 2)?;
        Ok(s(input
            .chars()
            .skip(start)
            .take(end.saturating_sub(start))
            .collect::<String>()))
    }
}

#[derive(Clone, Copy)]
pub struct Length;
impl Op for Length {
    fn call(
        &self,
        args: &[serde_json::Value],
        _ctx: &mut dyn OpContext,
    ) -> Result<serde_json::Value, OpError> {
        let input = require_one_string(args)?;
        Ok(num(input.chars().count() as f64))
    }
}

#[derive(Clone, Copy)]
pub struct Concat;
impl Op for Concat {
    fn call(
        &self,
        args: &[serde_json::Value],
        _ctx: &mut dyn OpContext,
    ) -> Result<serde_json::Value, OpError> {
        let a = args
            .first()
            .and_then(|v| v.as_str())
            .ok_or_else(|| type_mismatch("string", "non-string"))?;
        let b = args
            .get(1)
            .and_then(|v| v.as_str())
            .ok_or_else(|| type_mismatch("string", "non-string"))?;
        Ok(s(format!("{a}{b}")))
    }
}

#[derive(Clone, Copy)]
pub struct CodePointAt;
impl Op for CodePointAt {
    fn call(
        &self,
        args: &[serde_json::Value],
        _ctx: &mut dyn OpContext,
    ) -> Result<serde_json::Value, OpError> {
        let input = require_one_string(args)?;
        let index = require_char_index(args, 1)?;
        let char_count = input.chars().count();
        input
            .chars()
            .nth(index)
            .map(|c| num(c as u32 as f64))
            .ok_or_else(|| {
                OpError::Other(format!(
                    "index {index} out of bounds (char count: {char_count})"
                ))
            })
    }
}

// ---------------------------------------------------------------------------
// Bulk char-sequence conversion atoms (O(n) split/join)
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
pub struct Chars;
impl Op for Chars {
    fn call(
        &self,
        args: &[serde_json::Value],
        _ctx: &mut dyn OpContext,
    ) -> Result<serde_json::Value, OpError> {
        let input = require_one_string(args)?;
        let chars: Vec<serde_json::Value> = input
            .chars()
            .map(|c| serde_json::Value::String(c.to_string()))
            .collect();
        Ok(serde_json::Value::Array(chars))
    }
}

#[derive(Clone, Copy)]
pub struct FromChars;
impl Op for FromChars {
    fn call(
        &self,
        args: &[serde_json::Value],
        _ctx: &mut dyn OpContext,
    ) -> Result<serde_json::Value, OpError> {
        let arg = args.first().ok_or_else(|| type_mismatch("1 args", "0"))?;
        let arr = arg
            .as_array()
            .ok_or_else(|| type_mismatch("array", "non-array"))?;
        let mut result = String::new();
        for v in arr {
            result.push_str(require_string(v)?);
        }
        Ok(s(result))
    }
}

// ---------------------------------------------------------------------------
// Op Registry
// ---------------------------------------------------------------------------

/// Build the [`OpRegistry`] for `char`, registering the Rust handls under
/// their qualified op names (`"char.<fn>"` — see [`CHAR_OPS`]).
pub fn char_op_registry() -> OpRegistry {
    let mut registry = OpRegistry::new();
    registry.register("char.is_whitespace", Arc::new(IsWhitespace));
    registry.register("char.is_alpha", Arc::new(IsAlpha));
    registry.register("char.is_numeric", Arc::new(IsNumeric));
    registry.register("char.is_alnum", Arc::new(IsAlnum));
    registry.register("char.is_uppercase", Arc::new(IsUppercase));
    registry.register("char.is_lowercase", Arc::new(IsLowercase));
    registry.register("char.to_upper", Arc::new(ToUpper));
    registry.register("char.to_lower", Arc::new(ToLower));
    registry.register("char.to_title", Arc::new(ToTitle));
    registry.register("char.code_point", Arc::new(CodePoint));
    registry.register("char.from_code_point", Arc::new(FromCodePoint));
    registry.register("char.at", Arc::new(At));
    registry.register("char.slice", Arc::new(Slice));
    registry.register("char.length", Arc::new(Length));
    registry.register("char.concat", Arc::new(Concat));
    registry.register("char.code_point_at", Arc::new(CodePointAt));
    registry.register("char.chars", Arc::new(Chars));
    registry.register("char.from_chars", Arc::new(FromChars));
    registry
}
