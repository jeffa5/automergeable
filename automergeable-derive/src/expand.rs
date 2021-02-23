use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{Data, DataStruct, DeriveInput, Fields};

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

        quote! {
            fields.insert(#field_name_string.to_owned(), self.#field_name.to_automerge());
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
