use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{Attribute, Data, DataStruct, DeriveInput, Fields, Lit, Meta, NestedMeta};

pub(crate) fn from_automerge(input: DeriveInput) -> TokenStream {
    let fields = match input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => fields.named,
        _ => panic!("this derive macro only works on structs with named fields"),
    };
    let t_name = input.ident;
    let from_automerge_fields = fields.iter().map(|f| {
        let field_name = f.ident.as_ref().unwrap();
        let field_ty = &f.ty;

        let repr = get_representation_type(&f.attrs, field_name, field_ty);
        quote! {
            #field_name: #repr,
        }
    });
    quote! {
        #[automatically_derived]
        impl ::automergeable_traits::FromAutomerge for #t_name {
            fn from_automerge(value: &::automerge::Value) -> ::std::result::Result<Self, ::automergeable_traits::FromAutomergeError> {
                if let ::automerge::Value::Map(hm, ::automerge::MapType::Map) = value {
                    Ok(Self {
                        #(#from_automerge_fields)*
                    })
                } else {
                    Err(::automergeable_traits::FromAutomergeError::WrongType)
                }
            }
        }
    }
}

fn get_representation_type(
    attrs: &[Attribute],
    field_name: &Ident,
    field_ty: &syn::Type,
) -> TokenStream {
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
    let field_name_string = format_ident!("{}", field_name).to_string();
    let value_for_field = quote! {
        hm.get(#field_name_string).ok_or(::automergeable_traits::FromAutomergeError::WrongType)?
    };
    match ty.as_deref() {
        Some("Text") => {
            quote! { <::std::vec::Vec<char>>::from_automerge(#value_for_field)?.into_iter().collect() }
        }
        Some("Counter") => {
            quote! { if let ::automerge::Value::Primitive(::automerge::ScalarValue::Counter(i)) = #value_for_field {
                *i
            } else {
                return Err(::automergeable_traits::FromAutomergeError::WrongType)
            }}
        }
        Some("Timestamp") => {
            quote! { if let ::automerge::Value::Primitive(::automerge::ScalarValue::Timestamp(i)) = #value_for_field {
                *i
            } else {
                return Err(::automergeable_traits::FromAutomergeError::WrongType)
            }}
        }
        _ => {
            quote! { <#field_ty>::from_automerge(#value_for_field)? }
        }
    }
}
