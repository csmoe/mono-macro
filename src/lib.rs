//! Mono macro
//! ==================
//!
//! This crate provides the `#[mono]` macro to force a generic function to be monomorphizied with given types.
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
//! Since we are monomorphizing ourselves, you are required to spell out the static dispatch manually:
//!
//! In a bare function case,
//! ```rust
//! use mono_macro::mono;
//! #[mono(T = i32, U = i64)]
//! fn func<T, U>(t: T, u: U) {}
//! ```
//!
//! it will be expanded to:
//! ```rust
//! pub const _: *const () = (&func::<i32, i64>) as *const _ as _;
//! fn func<T, U>(t: T, u: U) {}
//! ```
//!
//! For more complicated case, use `mono_macro!` instead:
//! ```rust
//! trait Tr<T> {
//!     fn foo(&self, _t: T) {}
//! }
//!
//! struct Foo<'a> {
//!     t: &'a str,
//! }
//!
//! impl<'a, T> Tr<T> for Foo<'a> {
//!     fn foo(&self, _t: T) {}
//! }
//!
//! mono_macro!(<Foo<'static> as Tr<i32>>::foo);
//! ```
//!
//! this will expand to:
//! ```rust
//! trait Tr<T> {
//!     fn foo(&self, _t: T) {}
//! }
//!
//! struct Foo<'a> {
//!     t: &'a str,
//! }
//!
//! impl<'a, T> Tr<T> for Foo<'a> {
//!     fn foo(&self, _t: T) {}
//! }
//!
//! pub const _: *const () = (&<Foo<'static> as Tr<i32>>::foo) as *const _ as _;
//! ```

use proc_macro::TokenStream;
use quote::quote;
use syn::parse::Parse;
use syn::parse::ParseStream;
use syn::parse_macro_input;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::GenericParam;
use syn::Ident;
use syn::ItemFn;
use syn::Lifetime;
use syn::Token;
use syn::TypePath;

/// Apply this macro on a generic function will cast the **bare function** pointer with given type into pointer, which forces this function to be monomorphized.
///
/// # Example
/// ```rust,no_run
/// use mono_macro::mono;
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
    let fn_span = fn_sig.span();
    let func_ident = fn_sig.ident.clone();

    let mut params = vec![];
    for g in fn_sig.generics.params.into_iter() {
        if let Some(t) = mono_attr
            .eqs
            .iter()
            .find(|eq| match (&g, &eq.type_or_lifetime) {
                // (GenericParam::Lifetime(ld), TypeOrLifetime::Lifetime(l)) => &ld.lifetime == l,
                (GenericParam::Type(t1), TypeOrLifetime::Type(t2)) => &t1.ident == t2,
                (_, _) => false,
            })
        {
            params.push(t.param.clone());
        } else if matches!(g, GenericParam::Type(_)) {
            let err = syn::Error::new(fn_span, "all the type parameters should be spelled out")
                .into_compile_error()
                .into();
            return err;
        }
    }

    let mut expand = TokenStream::from(quote! {
        pub const _: *const () = (&#func_ident::<#(#params,)*>) as *const _ as _;
    });

    expand.extend(func);
    expand
}

/// Force monomorphizing on a path of function, for the complex functions like impl methods of generic types.
/// For example,
/// ```rust,no_run
/// use mono_macro::mono_macro;
/// struct Foo<T>(T);
/// trait Trait<K> {
///     fn method(&self, k: K);
/// }
/// impl<T, K> Trait<K> for Foo<T> {
///     fn method(&self, k: K) {}
/// }
///
/// mono_macro!(<Foo<i32> as Trait<u8>>::method);
/// ```
///
/// this will expand to:
/// ```rust,no_run
/// use mono_macro::mono_macro;
/// struct Foo<T>(T);
/// trait Trait<K> {
///     fn method(&self, k: K);
/// }
/// impl<T, K> Trait<K> for Foo<T> {
///     fn method(&self, k: K) {}
/// }
/// pub const _: *const () = (&<Foo<i32> as Trait<u8>>::method) as *const _ as _;
/// ```
#[proc_macro]
pub fn mono_macro(input: TokenStream) -> TokenStream {
    let path = parse_macro_input!(input as TypePath);
    TokenStream::from(quote! {
        pub const _: *const () = (&#path) as *const _ as _;
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
    type_or_lifetime: TypeOrLifetime,
    #[allow(dead_code)]
    eq_token: Token![=],
    param: TypeOrLifetime,
}

impl Parse for TypeEqTo {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(TypeEqTo {
            type_or_lifetime: input.parse()?,
            eq_token: input.parse()?,
            param: input.parse()?,
        })
    }
}

#[derive(Clone, Debug)]
enum TypeOrLifetime {
    Type(Ident),
    Lifetime(syn::Lifetime),
}

impl Parse for TypeOrLifetime {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Lifetime) {
            input.parse().map(TypeOrLifetime::Lifetime)
        } else if lookahead.peek(Ident) {
            input.parse().map(TypeOrLifetime::Type)
        } else {
            Err(lookahead.error())
        }
    }
}

extern crate proc_macro2;
use proc_macro2::TokenStream as TokenStream2;
impl quote::ToTokens for TypeOrLifetime {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self {
            TypeOrLifetime::Lifetime(l) => l.to_tokens(tokens),
            TypeOrLifetime::Type(t) => t.to_tokens(tokens),
        }
    }
}
