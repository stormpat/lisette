use crate::assert_emit_snapshot;

#[test]
fn array_literal() {
    let input = r#"
fn test() -> Array<int, 3> {
  [1, 2, 3]
}
"#;
    assert_emit_snapshot!(input);
}

#[test]
fn array_type_in_signature() {
    let input = r#"
fn first(xs: Array<int, 3>) -> int {
  xs[0]
}
"#;
    assert_emit_snapshot!(input);
}

#[test]
fn array_struct_field() {
    let input = r#"
struct Grid {
  cells: Array<int, 4>,
}

fn make() -> Grid {
  Grid { cells: [1, 2, 3, 4] }
}
"#;
    assert_emit_snapshot!(input);
}

#[test]
fn array_length() {
    let input = r#"
fn count(xs: Array<int, 3>) -> int {
  xs.length()
}
"#;
    assert_emit_snapshot!(input);
}

#[test]
fn array_equality() {
    let input = r#"
fn same(a: Array<int, 2>, b: Array<int, 2>) -> bool {
  a == b
}
"#;
    assert_emit_snapshot!(input);
}

#[test]
fn nested_array_type() {
    let input = r#"
fn grid() -> Array<Array<int, 3>, 2> {
  [[1, 2, 3], [4, 5, 6]]
}
"#;
    assert_emit_snapshot!(input);
}

#[test]
fn array_zero_value() {
    let input = r#"
struct Buf {
  data: Array<int, 4>,
}

fn empty() -> Buf {
  Buf { .. }
}
"#;
    assert_emit_snapshot!(input);
}

#[test]
fn array_new_turbofish() {
    let input = r#"
fn make() -> Array<int, 5> {
  Array.new<int, 5>()
}
"#;
    assert_emit_snapshot!(input);
}

#[test]
fn array_new_from_annotation() {
    let input = r#"
fn make() -> Array<int, 3> {
  let xs: Array<int, 3> = Array.new()
  xs
}
"#;
    assert_emit_snapshot!(input);
}
