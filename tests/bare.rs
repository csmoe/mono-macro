use mono_macro::mono;
use mono_macro::mono_macro;

#[test]
fn test_bare_fn() {
    #[mono(T = i32, U = String)]
    #[mono(T = u8, U = i32)]
    fn foo<T, U>(_t: T, _u: U) {}

    mono_macro!(foo::<i32, u8>);
}
