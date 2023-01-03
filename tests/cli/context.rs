use super::helpers::{fmn, spawn_test_daemon};
use anyhow::Result;
use predicates::str::diff;

#[test]
fn test_list_context() -> Result<()> {
    let guard = spawn_test_daemon("test_define_context")?;
    define_context("a");
    define_context("b");
    list_context(vec!["default", "a", "b"]);
    define_context("c");
    list_context(vec!["default", "a", "b", "c"]);
    let contexts = guard.read_contexts()?;
    assert_eq!(contexts.len(), 3 + 1);
    Ok(())
}

#[test]
fn test_switch_context() -> Result<()> {
    let _guard = spawn_test_daemon("test_define_context")?;
    define_context("foo");
    set_context("foo");
    list_context(vec!["foo", "default"]);
    Ok(())
}

#[test]
fn test_rm_context() -> Result<()> {
    let _guard = spawn_test_daemon("test_rm_context")?;
    define_context("foo");
    set_context("foo");
    rm_context("foo");
    list_context(vec!["default"]);
    Ok(())
}

fn define_context(context: &str) {
    fmn(&["context", "define", context]).assert().success();
}

fn set_context(context: &str) {
    fmn(&["context", "set", context]).assert().success();
}

fn list_context(context: Vec<&str>) {
    let output = format!(" * {}\n", context.join("\n   "));
    fmn(&["context", "list"]).assert().stdout(diff(output));
}

fn rm_context(context: &str) {
    fmn(&["context", "rm", context]).assert().success();
}
