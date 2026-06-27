use crate::spec::infer::*;

#[test]
fn array_literal_with_annotation() {
    infer("let xs: Array<int, 3> = [1, 2, 3]; xs").assert_last_type(array_type(3, int_type()));
}

#[test]
fn array_element_adapts_to_annotation() {
    infer("let xs: Array<int8, 2> = [1, 2]; xs").assert_last_type(array_type(2, int8_type()));
}

#[test]
fn array_index_returns_element() {
    infer("let xs: Array<int, 3> = [1, 2, 3]; xs[0]").assert_last_type(int_type());
}

#[test]
fn array_length_returns_int() {
    infer("let xs: Array<int, 3> = [1, 2, 3]; xs.length()").assert_last_type(int_type());
}

#[test]
fn array_equality_is_bool() {
    infer("let xs: Array<int, 2> = [1, 2]; let ys: Array<int, 2> = [3, 4]; xs == ys")
        .assert_last_type(bool_type());
}

#[test]
fn nested_array_type() {
    infer("let xs: Array<Array<int, 3>, 2> = [[1, 2, 3], [4, 5, 6]]; xs")
        .assert_last_type(array_type(2, array_type(3, int_type())));
}

#[test]
fn array_literal_too_few_elements() {
    infer("let xs: Array<int, 3> = [1, 2]; xs").assert_infer_code("array_literal_length_mismatch");
}

#[test]
fn array_literal_too_many_elements() {
    infer("let xs: Array<int, 2> = [1, 2, 3]; xs")
        .assert_infer_code("array_literal_length_mismatch");
}

#[test]
fn arrays_of_different_lengths_do_not_unify() {
    infer("let xs: Array<int, 3> = [1, 2, 3]; let ys: Array<int, 4> = xs; ys")
        .assert_infer_code("array_length_mismatch");
}

#[test]
fn array_element_type_mismatch() {
    infer(r#"let xs: Array<int, 2> = ["a", "b"]; xs"#).assert_infer_code("type_mismatch");
}

#[test]
fn array_size_must_be_literal() {
    infer("let xs: Array<int, int> = [1]; xs").assert_infer_code("array_size_not_literal");
}

#[test]
fn array_of_slices_is_not_comparable() {
    infer(
        "let xs: Array<Slice<int>, 2> = [[1], [2]]; let ys: Array<Slice<int>, 2> = [[3], [4]]; xs == ys",
    )
    .assert_infer_code("type_mismatch");
}
