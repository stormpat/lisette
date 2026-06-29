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

#[test]
fn array_new_with_turbofish() {
    infer("Array.new<int, 5>()").assert_type(array_type(5, int_type()));
}

#[test]
fn array_new_infers_size_from_annotation() {
    infer("let a: Array<int, 3> = Array.new(); a").assert_last_type(array_type(3, int_type()));
}

#[test]
fn array_new_without_size_errors() {
    infer("Array.new()").assert_infer_code("array_new_cannot_infer_size");
}

#[test]
fn array_new_non_literal_size_errors() {
    infer("Array.new<int, int>()").assert_infer_code("array_size_not_literal");
}

#[test]
fn array_new_wrong_arity_errors() {
    infer("Array.new<int>()").assert_infer_code("array_type_arity");
}

#[test]
fn array_new_rejects_value_arguments() {
    infer("Array.new<int, 3>(5)").assert_infer_code("array_new_takes_no_arguments");
}

#[test]
fn array_new_element_without_zero_errors() {
    infer("Array.new<Channel<int>, 2>()").assert_infer_code("array_new_no_zero");
}

#[test]
fn array_new_ref_element_without_zero_errors() {
    // `Ref<T>` has no zero; only an `Option<Ref<T>>` (zero = None) is fillable.
    infer("Array.new<Ref<int>, 2>()").assert_infer_code("array_new_no_zero");
}

#[test]
fn array_for_loop_binds_element_type() {
    // `_y: int = x` only type-checks if the loop variable is inferred as `int`.
    infer("let arr: Array<int, 3> = [1, 2, 3]; for x in arr { let _y: int = x }")
        .assert_no_errors();
}

#[test]
fn array_for_loop_element_type_mismatch() {
    infer("let arr: Array<int, 3> = [1, 2, 3]; for x in arr { let _y: string = x }")
        .assert_infer_code("type_mismatch");
}

#[test]
fn zero_length_array() {
    infer("let xs: Array<int, 0> = []; xs").assert_last_type(array_type(0, int_type()));
}

#[test]
fn integer_in_type_position_errors() {
    infer("let xs: Slice<3> = []; xs").assert_infer_code("int_literal_not_a_type");
}

#[test]
fn index_through_ref_deref() {
    infer("let arr: Array<int, 3> = [1, 2, 3]; let r = &arr; r.*[0]").assert_last_type(int_type());
}

#[test]
fn slice_of_arrays_element_is_array() {
    infer("let xs: Slice<Array<int, 3>> = [[1, 2, 3]]; xs[0]")
        .assert_last_type(array_type(3, int_type()));
}

#[test]
fn map_value_array_index_is_array() {
    infer("let m: Map<string, Array<int, 3>> = Map.new(); m[\"k\"]")
        .assert_last_type(array_type(3, int_type()));
}

#[test]
fn comparable_array_map_key_indexing() {
    infer("let m: Map<Array<int, 2>, string> = Map.new(); m[[1, 2]]")
        .assert_last_type(string_type());
}

#[test]
fn array_as_slice_returns_slice_of_element() {
    infer("let xs: Array<int, 3> = [1, 2, 3]; xs.as_slice()")
        .assert_last_type(slice_type(int_type()));
}
