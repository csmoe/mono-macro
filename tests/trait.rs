#[test]
fn test_trait_method() {
    trait Tr<T> {
        fn foo(&self, t: T) {}
    }

    struct Foo;

    impl<T> Tr<T> for Foo {
        fn foo(&self, t: T) {}
    }
}
