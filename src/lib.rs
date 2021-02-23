use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod expand;
use expand::expand_automergable;

#[proc_macro_derive(Automergable)]
pub fn automergable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_automergable(input).into()
}
