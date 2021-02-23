use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod expand;
use expand::expand_automergeable;

/// Derive the `Automergeable` trait.
///
/// Covers:
/// - conversion into an automerge `Value`
#[proc_macro_derive(Automergeable, attributes(automergeable))]
pub fn automergeable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_automergeable(input).into()
}
