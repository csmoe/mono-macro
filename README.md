Mono macro
==================

This crate provides the `#[mono]` macro to force a generic function to be monomorphizied with given types.

Pair with `share-generics` mode in rustc, this can result less code, for details see https://github.com/rust-lang/rust/pull/48779.

```toml
[dependencies]
mono-macro = "1.0"
```

<br>

## Usage

Since we are monomorphizing ourselves, you are required to spell out the static dispatch manually:

In a bare function case,
```rust
#[mono(T = i32, U = i64)]
fn func<T, U>(t: T, u: U) {
    ...
}
```

it will be expanded to:
```rust
pub const _: *const () = (&foo::<i32, i64>) as *const _ as _;
fn func<T, U>(t: T, u: U) {
    ...
}
```

## TODO
* [ ] impl methods

* [ ] function like macro for complicated functions

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
