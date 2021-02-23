use proc_macro2::TokenStream;
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
        let field_name_path = format_ident!("{}_path", field_name);
        let path_key = format_ident!("{}", field_name).to_string();

        quote! {
            pub fn #field_name_path() -> ::automerge::Path {
                ::automerge::Path::root().key(#path_key)
            }
        }
    });
    let field_diffs = fields.iter().map(|f| {
        let field_name = f.ident.as_ref().unwrap();
        let path_key = format_ident!("{}", field_name).to_string();
        quote!{
            changes.append(&mut self.#field_name.diff(path.clone().key(#path_key), &original.#field_name));
        }
    });
    quote! {
        #[automatically_derived]
        impl #t_name {
            #(#imp_paths)*
        }

        #[automatically_derived]
        impl ::automergeable_core::ToValue for #t_name {
            fn to_value(&self) -> ::automerge::Value {
                todo!()
            }
        }

        #[automatically_derived]
        impl ::automergeable_core::AutoDiff for #t_name {
            fn diff(&self, path: ::automerge::Path, original: &Self) -> ::std::vec::Vec<::automerge::LocalChange> {
                let mut changes = ::std::vec::Vec::new();
                #(#field_diffs)*
                changes
            }
        }
    }
}
