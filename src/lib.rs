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

/// Apply this macro on a generic function will cast the function pointer with given type into pointer, which forces this function to be monomorphized.
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

    let mut params = vec![];
    let generics_num = fn_sig.generics.params.len();
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

    let func_ident = fn_sig.ident.clone();
    let mut expand = force_monomorphize(func_ident, params);
    expand.extend(func);
    expand
}

fn force_monomorphize(func: Ident, ident: Vec<TypeOrLifetime>) -> TokenStream {
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
