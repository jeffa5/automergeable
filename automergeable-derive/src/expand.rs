use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{Attribute, Data, DataStruct, DeriveInput, Fields, Lit, Meta, NestedMeta};

pub fn expand_automergeable(input: DeriveInput) -> TokenStream {
    let fields = match input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => fields.named,
        _ => panic!("this derive macro only works on structs with named fields"),
    };
    let t_name = input.ident;
    let imp_paths = fields.iter().map(|f| {
        let field_name = f.ident.as_ref().unwrap();
        let field_name_path = Ident::new(
            &format!("{}_PATH_KEY", field_name.to_string().to_uppercase()),
            field_name.span(),
        );
        let path_key = format_ident!("{}", field_name).to_string();

        quote! {
            const #field_name_path : &'static str = #path_key;
        }
    });
    let to_automerge_fields = fields.iter().map(|f| {
        let field_name = f.ident.as_ref().unwrap();
        let field_name_string = format_ident!("{}", field_name).to_string();

        let repr = get_representation_type(&f.attrs, field_name);
        quote! {
            fields.insert(#field_name_string.to_owned(), #repr);
        }
    });
    quote! {
        #[automatically_derived]
        impl #t_name {
            #(#imp_paths)*
        }

        #[automatically_derived]
        impl ::automergeable_core::ToAutomerge for #t_name {
            fn to_automerge(&self) -> ::automerge::Value {
                let mut fields = ::std::collections::HashMap::new();
                #(#to_automerge_fields)*
                ::automerge::Value::Map(fields, ::automerge::MapType::Map)
            }
        }
    }
}

fn get_representation_type(attrs: &[Attribute], field_name: &Ident) -> TokenStream {
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
