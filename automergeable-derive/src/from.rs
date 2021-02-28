use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    punctuated::Punctuated, token::Comma, Attribute, Data, DataStruct, DeriveInput, Field, Fields,
    Lit, Meta, NestedMeta, Type,
};

pub(crate) fn from_automerge(input: DeriveInput) -> TokenStream {
    match &input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => from_automerge_struct_named_fields(&input, &fields.named),
        Data::Struct(DataStruct {
            fields: Fields::Unnamed(fields),
            ..
        }) => from_automerge_struct_unnamed_fields(&input, &fields.unnamed),
        _ => panic!("this derive macro only works on structs with named fields"),
    }
}

fn from_automerge_struct_named_fields(
    input: &DeriveInput,
    fields: &Punctuated<Field, Comma>,
) -> TokenStream {
    let t_name = &input.ident;
    let from_automerge_fields = fields.iter().map(|f| {
        let field_name = f.ident.as_ref().unwrap();
        let field_ty = &f.ty;

        let field_name_string = format_ident!("{}", field_name).to_string();
        let value_for_field = quote! {
            hm.get(#field_name_string)
        };
        let repr = get_representation_type(&f.attrs, field_ty, value_for_field);
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
                    Err(::automergeable_traits::FromAutomergeError::WrongType {
                        found: value.clone(),
                    })
                }
            }
        }
    }
}

fn from_automerge_struct_unnamed_fields(
    input: &DeriveInput,
    fields: &Punctuated<Field, Comma>,
) -> TokenStream {
    let t_name = &input.ident;
    let t_name_string = format_ident!("{}", t_name).to_string();
    let from_automerge_fields = fields.iter().enumerate().map(|(i, f)| {
        let field_name = syn::Index::from(i);
        let field_ty = &f.ty;

        let value_for_field = quote! {
            seq.get(#field_name)
        };
        let repr = get_representation_type(&f.attrs, field_ty, value_for_field);
        quote! {
            #repr,
        }
    });
    quote! {
        #[automatically_derived]
        impl ::automergeable_traits::FromAutomerge for #t_name {
            fn from_automerge(value: &::automerge::Value) -> ::std::result::Result<Self, ::automergeable_traits::FromAutomergeError> {
                if let ::automerge::Value::Map(hm, ::automerge::MapType::Map) = value {
                    if let Some(value) = hm.get(#t_name_string) {
                        if let ::automerge::Value::Sequence(seq) = value {
                            Ok(Self(
                                #(#from_automerge_fields)*
                            ))
                        } else {
                            Err(::automergeable_traits::FromAutomergeError::WrongType {
                                found: value.clone(),
                            })
                        }
                    } else {
                        Err(::automergeable_traits::FromAutomergeError::WrongType {
                            found: value.clone(),
                        })
                    }
                } else {
                    Err(::automergeable_traits::FromAutomergeError::WrongType {
                        found: value.clone(),
                    })
                }
            }
        }
    }
}

fn get_representation_type(
    attrs: &[Attribute],
    field_ty: &Type,
    value_for_field: TokenStream,
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
    match ty.as_deref() {
        Some("Text") => {
            quote! {
                if let Some(value) = #value_for_field {
                    <::std::vec::Vec<char>>::from_automerge(value)?.into_iter().collect()
                } else {
                    <#field_ty>::default()
                }
            }
        }
        Some("Counter") => {
            quote! {
                if let Some(value) = #value_for_field {
                    if let ::automerge::Value::Primitive(::automerge::ScalarValue::Counter(i)) = value {
                    *i
                    } else {
                        return Err(::automergeable_traits::FromAutomergeError::WrongType {
                            found: value.clone(),
                        })
                    }
                } else {
                    <#field_ty>::default()
                }
            }
        }
        Some("Timestamp") => {
            quote! {
                if let Some(value) = #value_for_field {
                    if let ::automerge::Value::Primitive(::automerge::ScalarValue::Timestamp(i)) = value {
                        *i
                    } else {
                        return Err(::automergeable_traits::FromAutomergeError::WrongType {
                            found: value.clone(),
                        })
                    }
                } else {
                    <#field_ty>::default()
                }
            }
        }
        _ => {
            quote! {
                if let Some(value) = #value_for_field {
                    <#field_ty>::from_automerge(value)?
                } else {
                    <#field_ty>::default()
                }
            }
        }
    }
}
