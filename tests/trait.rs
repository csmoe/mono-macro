use mono_macro::mono_macro;
#[test]
fn test_trait_method() {
    trait Tr<T> {
        fn foo(&self, _t: T) {}
    }

    struct Foo<'a> {
        t: &'a str,
    }

    impl<'a, T> Tr<T> for Foo<'a> {
        fn foo(&self, _t: T) {}
    }

    mono_macro!(<Foo<'static> as Tr<i32>>::foo);
}
