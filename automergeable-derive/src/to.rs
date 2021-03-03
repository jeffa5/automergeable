use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use syn::{
    punctuated::Punctuated, token::Comma, Attribute, Data, DataEnum, DataStruct, DeriveInput,
    Fields, Lit, Meta, NestedMeta, Variant,
};

use crate::utils;

pub(crate) fn to_automerge(input: DeriveInput) -> TokenStream {
    match &input.data {
        Data::Struct(DataStruct { fields, .. }) => to_automerge_struct(&input, &fields),
        Data::Enum(DataEnum { variants, .. }) => to_automerge_enum(&input, &variants),
        _ => panic!("this derive macro only works on structs with named fields"),
    }
}

fn to_automerge_struct(input: &DeriveInput, fields: &Fields) -> TokenStream {
    let traits_path = utils::traits_path(input);
    let t_name = &input.ident;
    let fields_to_automerge = fields_to_automerge(fields, true);
    quote! {
        #[automatically_derived]
        impl #traits_path::ToAutomerge for #t_name {
            fn to_automerge(&self) -> ::automerge::Value {
                #fields_to_automerge
            }
        }
    }
}

fn to_automerge_enum(input: &DeriveInput, variants: &Punctuated<Variant, Comma>) -> TokenStream {
    let traits_path = utils::traits_path(input);
    let t_name = &input.ident;
    let variants = variants.iter().map(|v| {
        let v_name = &v.ident;
        let fields = match &v.fields {
            Fields::Named(n) => {
                let names = n.named.iter().map(|n| {
                    let name = &n.ident;
                    quote! { #name, }
                });
                quote! {{
                    #(#names)*
                }}
            }
            Fields::Unnamed(u) => {
                let items = u.unnamed.iter().enumerate().map(|(i, _)| {
                    let a = Ident::new(&format!("f{}", i), Span::call_site());
                    quote! { #a, }
                });
                quote! {( #(#items)* )}
            }
            Fields::Unit => {
                quote! {}
            }
        };
        let fields_to_automerge = fields_to_automerge(&v.fields, false);
        let v_name_string = v_name.to_string();
        quote! {
            Self::#v_name#fields => {
                let mut outer = ::std::collections::HashMap::new();
                let fields = {#fields_to_automerge};
                outer.insert(#v_name_string.to_owned(), fields);
                ::automerge::Value::Map(outer, ::automerge::MapType::Map)
            }
        }
    });
    quote! {
        #[automatically_derived]
        impl #traits_path::ToAutomerge for #t_name {
            fn to_automerge(&self) -> ::automerge::Value {
                match self {
                    #(#variants)*
                }
            }
        }
    }
}

fn get_representation_type(attrs: &[Attribute], field_name: TokenStream) -> TokenStream {
    let mut ty = None;
    for a in attrs {
        match a.parse_meta().unwrap() {
            Meta::NameValue(_) => {}
            Meta::List(meta) => {
                if Some("automergeable".to_owned()) == meta.path.get_ident().map(|i| i.to_string())
                {
                    for m in meta.nested {
                        match m {
                            NestedMeta::Meta(meta) => match meta {
                                Meta::Path(_) | Meta::List(_) => {}
                                Meta::NameValue(n) => {
                                    if let Lit::Str(lit) = &n.lit {
                                        ty = Some(lit.value())
                                    }
                                }
                            },
                            NestedMeta::Lit(Lit::Str(_)) => {}
                            _ => {}
                        }
                    }
                }
            }
            Meta::Path(_) => {}
        }
    }
    match ty.as_deref() {
        Some("Text") => {
            quote! { ::automerge::Value::Text(#field_name.chars().collect::<::std::vec::Vec<_>>()) }
        }
        Some("Counter") => {
            quote! { ::automerge::Value::Primitive(::automerge::Primitive::Counter(#field_name)) }
        }
        Some("Timestamp") => {
            quote! { ::automerge::Value::Primitive(::automerge::Primitive::Timestamp(#field_name)) }
        }
        _ => quote! { #field_name.to_automerge() },
    }
}

fn fields_to_automerge(fields: &Fields, is_struct: bool) -> TokenStream {
    match fields {
        Fields::Named(n) => {
            let fields = n.named.iter().map(|f| {
                let field_name = f.ident.as_ref().unwrap();
                let field_name_string = format_ident!("{}", field_name).to_string();

                let field_name = if is_struct {
                    quote! {self.#field_name}
                } else {
                    quote! {#field_name}
                };
                let repr = get_representation_type(&f.attrs, field_name);
                quote! {
                    fields.insert(#field_name_string.to_owned(), #repr);
                }
            });
            quote! {
                let mut fields = ::std::collections::HashMap::new();
                #(#fields)*
                ::automerge::Value::Map(fields, ::automerge::MapType::Map)
            }
        }
        Fields::Unnamed(u) => {
            let fields = u.unnamed.iter().enumerate().map(|(i, f)| {
                let field_name = if is_struct {
                    let field_name = syn::Index::from(i);
                    quote! {self.#field_name}
                } else {
                    let f = Ident::new(&format!("f{}", i), Span::call_site());
                    quote! {#f}
                };
                let repr = get_representation_type(&f.attrs, field_name);
                quote! {
                    fields.push(#repr);
                }
            });
            quote! {
                let mut fields = Vec::new();
                #(#fields)*
                ::automerge::Value::Sequence(fields)
            }
        }
        Fields::Unit => {
            quote! {
                ::automerge::Value::Primitive(::automerge::Primitive::Null)
            }
        }
    }
}
