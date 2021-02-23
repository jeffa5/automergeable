use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod expand;
use expand::expand_automergeable;

#[proc_macro_derive(Automergeable)]
pub fn automergeable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_automergeable(input).into()
}
