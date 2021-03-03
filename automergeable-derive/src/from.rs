use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    punctuated::Punctuated, token::Comma, Attribute, Data, DataEnum, DataStruct, DeriveInput,
    Fields, Ident, Lit, Meta, NestedMeta, Type, Variant,
};

use crate::utils;

pub(crate) fn from_automerge(input: DeriveInput) -> TokenStream {
    match &input.data {
        Data::Struct(DataStruct { fields, .. }) => from_automerge_struct(&input, &fields),
        Data::Enum(DataEnum { variants, .. }) => from_automerge_enum(&input, &variants),
        _ => panic!("this derive macro only works on structs"),
    }
}

fn from_automerge_struct(input: &DeriveInput, fields: &Fields) -> TokenStream {
    let traits_path = utils::traits_path(input);
    let t_name = &input.ident;
    let fields_from_automerge = fields_from_automerge(fields, None, &traits_path);
    quote! {
        #[automatically_derived]
        impl #traits_path::FromAutomerge for #t_name {
            fn from_automerge(value: &::automerge::Value) -> ::std::result::Result<Self, #traits_path::FromAutomergeError> {
                #fields_from_automerge
            }
        }
    }
}

fn from_automerge_enum(input: &DeriveInput, variants: &Punctuated<Variant, Comma>) -> TokenStream {
    let traits_path = utils::traits_path(input);
    let t_name = &input.ident;
    let variant_match = variants.iter().map(|v| {
        let v_name = &v.ident;
        let v_name_string = v_name.to_string();
        let fields_from_automerge =
            fields_from_automerge(&v.fields, Some(v_name.clone()), &traits_path);
        quote! {
            (#v_name_string, value) => {#fields_from_automerge}
        }
    });
    quote! {
        #[automatically_derived]
        impl #traits_path::FromAutomerge for #t_name {
            fn from_automerge(value: &::automerge::Value) -> ::std::result::Result<Self, #traits_path::FromAutomergeError> {
                if let ::automerge::Value::Map(hm, ::automerge::MapType::Map) = value {
                    if hm.len() != 1 {
                        Err(#traits_path::FromAutomergeError::WrongType {
                            found: value.clone(),
                        })
                    } else {
                        match hm.iter().map(|(k,v)| (k.as_str(), v)).next().unwrap() {
                            #(#variant_match)*
                            _ => Err(#traits_path::FromAutomergeError::WrongType {
                                found: value.clone(),
                            })
                        }
                    }
                } else {
                    Err(#traits_path::FromAutomergeError::WrongType {
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
    traits_path: &TokenStream,
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
                    if let ::automerge::Value::Primitive(::automerge::Primitive::Counter(i)) = value {
                    *i
                    } else {
                        return Err(#traits_path::FromAutomergeError::WrongType {
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
                    if let ::automerge::Value::Primitive(::automerge::Primitive::Timestamp(i)) = value {
                        *i
                    } else {
                        return Err(#traits_path::FromAutomergeError::WrongType {
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

fn fields_from_automerge(
    fields: &Fields,
    variant_name: Option<Ident>,
    traits_path: &TokenStream,
) -> TokenStream {
    let ty_name = if let Some(name) = variant_name {
        quote! {Self::#name}
    } else {
        quote! {Self}
    };
    match fields {
        Fields::Named(n) => {
            let fields = n.named.iter().map(|f| {
                let field_name = f.ident.as_ref().unwrap();
                let field_ty = &f.ty;

                let field_name_string = format_ident!("{}", field_name).to_string();
                let value_for_field = quote! {
                    hm.get(#field_name_string)
                };
                let repr =
                    get_representation_type(&f.attrs, field_ty, value_for_field, traits_path);
                quote! {
                    #field_name: #repr,
                }
            });
            quote! {
                if let ::automerge::Value::Map(hm, ::automerge::MapType::Map) = value {
                    Ok(#ty_name {
                        #(#fields)*
                    })
                } else {
                    Err(#traits_path::FromAutomergeError::WrongType {
                        found: value.clone(),
                    })
                }
            }
        }
        Fields::Unnamed(u) => {
            let fields = u.unnamed.iter().enumerate().map(|(i, f)| {
                let field_name = syn::Index::from(i);
                let field_ty = &f.ty;

                let value_for_field = quote! {
                    seq.get(#field_name)
                };
                let repr =
                    get_representation_type(&f.attrs, field_ty, value_for_field, traits_path);
                quote! {
                    #repr,
                }
            });
            quote! {
                if let ::automerge::Value::Sequence(seq) = value {
                    Ok(#ty_name(
                        #(#fields)*
                    ))
                } else {
                    Err(#traits_path::FromAutomergeError::WrongType {
                        found: value.clone(),
                    })
                }
            }
        }
        Fields::Unit => {
            quote! {
                if let ::automerge::Value::Primitive(::automerge::Primitive::Null) = value {
                    Ok(#ty_name)
                } else {
                    Err(#traits_path::FromAutomergeError::WrongType {
                        found: value.clone(),
                    })
                }
            }
        }
    }
}
