use mono_macro::mono;

#[test]
fn test_lifetime() {
    #[mono(T = i32)]
    fn foo<'a, T>(_s: &'a str, _t: T) {}
}
