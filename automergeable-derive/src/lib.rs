use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

mod from;
mod to;

/// Derive the `Automergeable` trait.
///
/// Covers:
/// - conversion into an automerge `Value`
#[proc_macro_derive(Automergeable, attributes(automergeable))]
pub fn automergeable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let to = to::to_automerge(input.clone());
    let from = from::from_automerge(input);
    (quote! {
        #to
        #from
    })
    .into()
}

/// Derive the `ToAutomerge` trait.
#[proc_macro_derive(ToAutomerge, attributes(automergeable))]
pub fn to_automerge(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    to::to_automerge(input).into()
}

/// Derive the `FromAutomerge` trait.
#[proc_macro_derive(FromAutomerge, attributes(automergeable))]
pub fn from_automerge(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    from::from_automerge(input).into()
}
