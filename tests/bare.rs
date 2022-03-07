use mono_macro::mono;

#[test]
fn test_bare_fn() {
    #[mono(T = i32, U = String)]
    #[mono(T = u8, U = i32)]
    fn foo<T, U>(_t: T, _u: U) {}
}
