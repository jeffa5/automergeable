use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod to;

/// Derive the `Automergeable` trait.
///
/// Covers:
/// - conversion into an automerge `Value`
#[proc_macro_derive(Automergeable, attributes(automergeable))]
pub fn automergeable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    to::to_automerge(input).into()
}

#[proc_macro_derive(ToAutomerge, attributes(automergeable))]
pub fn to_automerge(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    to::to_automerge(input).into()
}
