use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{punctuated::Punctuated, DeriveInput};

pub(crate) fn crate_path(input: &DeriveInput) -> TokenStream {
    let path = input.attrs.iter().find_map(|a| {
        a.path.get_ident().map(|i| i.to_string()).and_then(|i| {
            if i == "automergeable" {
                a.parse_meta().ok().and_then(|m| {
                    if let syn::Meta::List(l) = m {
                        l.nested.iter().find_map(|n| {
                            if let syn::NestedMeta::Meta(syn::Meta::NameValue(n)) = n {
                                if let (Some(path), syn::Lit::Str(lit)) =
                                    (n.path.get_ident().map(|i| i.to_string()), n.lit.clone())
                                {
                                    if path == "crate_path" {
                                        let mut path = syn::Path {
                                            leading_colon: None,
                                            segments: Punctuated::new(),
                                        };
                                        for segment in lit.value().split("::") {
                                            let ident = Ident::new(segment, lit.span());
                                            path.segments.push(syn::PathSegment {
                                                ident,
                                                arguments: syn::PathArguments::None,
                                            });
                                        }
                                        Some(quote! {#path})
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        })
                    } else {
                        None
                    }
                })
            } else {
                None
            }
        })
    });
    path.unwrap_or_else(|| quote! {automergeable})
}
