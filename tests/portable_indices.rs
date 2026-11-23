//! These cases must have the same results on 32-bit and 64-bit targets.
use regorus::{Engine, Value};

fn evaluate(expression: &str) -> anyhow::Result<Value> {
    let mut engine = Engine::new();
    engine.set_strict_builtin_errors(true);
    engine.add_policy(
        "test.rego".into(),
        format!("package test\nresult := {expression}"),
    )?;
    engine.eval_rule("data.test.result".into())
}

#[test]
fn substring_indices_do_not_wrap() -> anyhow::Result<()> {
    for (expression, expected) in [
        (r#"substring("a好😀z", 1, 2)"#, "好😀"),
        (r#"substring("abcd", 4294967296, 2)"#, ""),
        (r#"substring("abcd", 4294967295 + 1, 2)"#, ""),
        (r#"substring("abcd", 9223372036854775807, 2)"#, ""),
        (r#"substring("a好😀z", 0, 4294967296)"#, "a好😀z"),
        (r#"substring("a好😀z", 1, 9223372036854775807)"#, "好😀z"),
        (r#"substring("a好😀z", 1, -1)"#, "好😀z"),
        (r#"substring("abcd", 1, 0)"#, ""),
        (r#"substring("abcd", 1.5, 2)"#, ""),
    ] {
        assert_eq!(evaluate(expression)?, Value::from(expected), "{expression}");
    }
    assert!(evaluate(r#"substring("abcd", -1, 2)"#).is_err());
    Ok(())
}

#[test]
fn array_slice_clamps_before_pointer_width_conversion() -> anyhow::Result<()> {
    for (expression, expected) in [
        ("array.slice([0,1,2,3], 0, 4294967299)", "[0,1,2,3]"),
        ("array.slice([0,1,2,3], 4294967296, 4294967299)", "[]"),
        (
            "array.slice([0,1,2,3], 0, 9223372036854775807)",
            "[0,1,2,3]",
        ),
        ("array.slice([0,1,2,3], -1, 2)", "[0,1]"),
        ("array.slice([0,1,2,3], 0, -1)", "[]"),
        ("array.slice([0,1,2,3], 3, 1)", "[]"),
        ("array.slice([], 0, 4294967299)", "[]"),
    ] {
        assert_eq!(
            evaluate(expression)?,
            Value::from_json_str(expected)?,
            "{expression}"
        );
    }
    assert_eq!(evaluate("array.slice([0,1], 0, 1.5)")?, Value::Undefined);
    Ok(())
}

#[test]
fn array_lookup_never_truncates_a_numeric_key() -> anyhow::Result<()> {
    let array = Value::from_json_str("[11,22]")?;
    for index in [4294967296u64, 4294967297, i64::MAX as u64, u64::MAX] {
        assert_eq!(array[Value::from(index)], Value::Undefined, "{index}");
        assert_eq!(evaluate(&format!("[11,22][{index}]"))?, Value::Undefined);
    }
    assert_eq!(evaluate("[11,22][1]")?, Value::from(22));
    assert_eq!(evaluate("[11,22][-1]")?, Value::Undefined);
    assert_eq!(evaluate("{4294967296: 11}[4294967296]")?, Value::from(11));
    Ok(())
}
