//! Mono macro
//! ==================
//!
//! This crate provides the `#[mono]` macro to force a generic function to be monomorphizied with give types.
//!
//! Pair with `share-generics` mode in rustc, this can result less code, for details see <https://github.com/rust-lang/rust/pull/48779>.
//!
//! ```toml
//! [dependencies]
//! mono-macro = "1.0"
//! ```
//!
//! <br>
//!
//! ## Usage
//!
//! Since we are monomorphizing ourselves, you are required to spell out the static dispatch handly:
//!
//! In a bare function case,
//! ```rust
//! #[mono(T = i32, U = i64)]
//! fn func<T, U>(t: T, u: U) {
//!     ...
//! }
//! ```
//!
//! it will be expanded to:
//! ```rust
//! pub const _: *const () = (&foo::<i32, i64>) as *const _ as _;
//! fn func<T, U>(t: T, u: U) {
//!     ...
//! }
//! ```

use proc_macro::TokenStream;
use quote::quote;
use syn::parse::Parse;
use syn::parse::ParseStream;
use syn::parse_macro_input;
use syn::punctuated::Punctuated;
use syn::Ident;
use syn::ItemFn;
use syn::Token;
use syn::TypeParam;

/// Apply this macro on a generic function will cast the function pointer with given type into pointer, which forces this function to be monomorphized.
///
/// # Example
/// ```rust,no_run
/// #[mono(T = i32)]
/// fn foo<T>(t: T) {}
/// ```
/// expands to:
/// ```rust,no_run
/// pub const _: *const () = &foo::<i32> as *const _ as _;
/// fn foo<T>(t: T) {}
/// ```
///
#[proc_macro_attribute]
pub fn mono(attr: TokenStream, func: TokenStream) -> TokenStream {
    let mono_attr = parse_macro_input!(attr as TypeEqs);
    let input = func.clone();
    let fn_sig = parse_macro_input!(input as ItemFn).sig;
    let mut types = vec![];
    for g in fn_sig.generics.type_params() {
        if let Some(t) = mono_attr.eqs.iter().find(|eq| eq.ident == g.ident) {
            types.push(t.r#type.ident.clone());
        } else {
            break;
        }
    }
    let func_ident = fn_sig.ident.clone();

    let mut expand = force_monomorphize(func_ident, types);
    expand.extend(func);
    expand
}

fn force_monomorphize(func: Ident, ident: Vec<Ident>) -> TokenStream {
    TokenStream::from(quote! {
        pub const _: *const () = (&#func::<#(#ident,)*>) as *const _ as _;
    })
}

// T = i32, U = i64
struct TypeEqs {
    eqs: Punctuated<TypeEqTo, Token![,]>,
}

impl Parse for TypeEqs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(TypeEqs {
            eqs: { input.parse_terminated(TypeEqTo::parse)? },
        })
    }
}

// T = i32
struct TypeEqTo {
    ident: Ident,
    #[allow(dead_code)]
    eq_token: Token![=],
    r#type: TypeParam,
}

impl Parse for TypeEqTo {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(TypeEqTo {
            ident: input.parse()?,
            eq_token: input.parse()?,
            r#type: input.parse()?,
        })
    }
}
