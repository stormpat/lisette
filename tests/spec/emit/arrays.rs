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

#[test]
fn array_for_loop() {
    let input = r#"
fn sum(a: Array<int, 3>) -> int {
  let mut total = 0
  for x in a {
    total = total + x
  }
  total
}
"#;
    assert_emit_snapshot!(input);
}

#[test]
fn zero_length_array() {
    let input = r#"
fn empty() -> Array<int, 0> {
  []
}
"#;
    assert_emit_snapshot!(input);
}

#[test]
fn array_in_containers() {
    let input = r#"
type Addr = Array<byte, 4>

struct Holder {
  slice_of_arr: Slice<Array<int, 3>>,
  map_val_arr: Map<string, Array<int, 3>>,
  ptr_to_arr: Ref<Array<int, 3>>,
  multidim: Array<Array<int, 3>, 2>,
  aliased: Addr,
}
"#;
    assert_emit_snapshot!(input);
}

#[test]
fn array_as_map_key() {
    let input = r#"
fn count(m: Map<Array<int, 2>, string>) -> int {
  m.length()
}
"#;
    assert_emit_snapshot!(input);
}

// Zero values: primitive elements keep Go's `[N]T{}` zero-fill, but elements
// whose Lisette zero differs from Go's (e.g. `Option<T>`: None vs `Some(nil)`)
// must be filled per index.

#[test]
fn array_new_primitive_elements_use_go_zero_fill() {
    let input = r#"
fn ints() -> Array<int, 3> {
  Array.new<int, 3>()
}

fn bools() -> Array<bool, 2> {
  Array.new<bool, 2>()
}
"#;
    assert_emit_snapshot!(input);
}

#[test]
fn array_new_option_element_fills_with_none() {
    let input = r#"
fn opts() -> Array<Option<int>, 2> {
  Array.new<Option<int>, 2>()
}
"#;
    assert_emit_snapshot!(input);
}

#[test]
fn array_new_struct_with_option_field_fills() {
    let input = r#"
struct Horse {
  speed: int,
  fast: Option<int>,
}

fn herd() -> Array<Horse, 2> {
  Array.new<Horse, 2>()
}
"#;
    assert_emit_snapshot!(input);
}

#[test]
fn array_zero_value_struct_field_with_option_element() {
    let input = r#"
struct Horse {
  speed: int,
  fast: Option<int>,
}

struct Stable {
  horses: Array<Horse, 2>,
}

fn empty() -> Stable {
  Stable { .. }
}
"#;
    assert_emit_snapshot!(input);
}

#[test]
fn array_new_option_ref_element_fills_with_none() {
    let input = r#"
fn flags() -> Array<Option<Ref<bool>>, 2> {
  Array.new<Option<Ref<bool>>, 2>()
}
"#;
    assert_emit_snapshot!(input);
}

#[test]
fn array_as_slice_copies_into_new_slice() {
    let input = r#"
fn to_slice(a: Array<int, 3>) -> Slice<int> {
  a.as_slice()
}
"#;
    assert_emit_snapshot!(input);
}
