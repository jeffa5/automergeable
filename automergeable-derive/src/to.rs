use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{
    punctuated::Punctuated, token::Comma, Attribute, Data, DataStruct, DeriveInput, Field, Fields,
    Index, Lit, Meta, NestedMeta,
};

pub(crate) fn to_automerge(input: DeriveInput) -> TokenStream {
    match &input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => to_automerge_struct_named_fields(&input, &fields.named),
        Data::Struct(DataStruct {
            fields: Fields::Unnamed(fields),
            ..
        }) => to_automerge_struct_unnamed_fields(&input, &fields.unnamed),
        _ => panic!("this derive macro only works on structs with named fields"),
    }
}

fn to_automerge_struct_named_fields(
    input: &DeriveInput,
    fields: &Punctuated<Field, Comma>,
) -> TokenStream {
    let t_name = &input.ident;
    let to_automerge_fields = fields.iter().map(|f| {
        let field_name = f.ident.as_ref().unwrap();
        let field_name_string = format_ident!("{}", field_name).to_string();

        let repr = get_representation_type_named(&f.attrs, field_name);
        quote! {
            fields.insert(#field_name_string.to_owned(), #repr);
        }
    });
    quote! {
        #[automatically_derived]
        impl ::automergeable_traits::ToAutomerge for #t_name {
            fn to_automerge(&self) -> ::automerge::Value {
                let mut fields = ::std::collections::HashMap::new();
                #(#to_automerge_fields)*
                ::automerge::Value::Map(fields, ::automerge::MapType::Map)
            }
        }
    }
}

fn get_representation_type_named(attrs: &[Attribute], field_name: &Ident) -> TokenStream {
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
            quote! { ::automerge::Value::Text(self.#field_name.chars().collect::<::std::vec::Vec<_>>()) }
        }
        Some("Counter") => {
            quote! { ::automerge::Value::Primitive(::automerge::ScalarValue::Counter(self.#field_name)) }
        }
        Some("Timestamp") => {
            quote! { ::automerge::Value::Primitive(::automerge::ScalarValue::Timestamp(self.#field_name)) }
        }
        _ => quote! { self.#field_name.to_automerge() },
    }
}

fn to_automerge_struct_unnamed_fields(
    input: &DeriveInput,
    fields: &Punctuated<Field, Comma>,
) -> TokenStream {
    let t_name = &input.ident;
    let t_name_string = format_ident!("{}", &input.ident).to_string();
    let to_automerge_fields = fields.iter().enumerate().map(|(i, f)| {
        let field_name = syn::Index::from(i);

        let repr = get_representation_type_unnamed(&f.attrs, field_name);
        quote! {
            value.push(#repr);
        }
    });
    quote! {
        #[automatically_derived]
        impl ::automergeable_traits::ToAutomerge for #t_name {
            fn to_automerge(&self) -> ::automerge::Value {
                let mut fields = ::std::collections::HashMap::new();
                let mut value = Vec::new();
                #(#to_automerge_fields)*
                fields.insert(#t_name_string.to_owned(), ::automerge::Value::Sequence(value));
                ::automerge::Value::Map(fields, ::automerge::MapType::Map)
            }
        }
    }
}

fn get_representation_type_unnamed(attrs: &[Attribute], field_name: Index) -> TokenStream {
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
            quote! { ::automerge::Value::Text(self.#field_name.chars().collect::<::std::vec::Vec<_>>()) }
        }
        Some("Counter") => {
            quote! { ::automerge::Value::Primitive(::automerge::ScalarValue::Counter(self.#field_name)) }
        }
        Some("Timestamp") => {
            quote! { ::automerge::Value::Primitive(::automerge::ScalarValue::Timestamp(self.#field_name)) }
        }
        _ => quote! { self.#field_name.to_automerge() },
    }
}
