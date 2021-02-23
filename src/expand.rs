use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DataStruct, DeriveInput, Fields};

pub fn expand_automergable(input: DeriveInput) -> TokenStream {
    let fields = match input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => fields.named,
        _ => panic!("this derive macro only works on structs with named fields"),
    };
    let t_name = input.ident;
    let imp_paths = fields.into_iter().map(|f| {
        let field_name = f.ident.unwrap();
        let field_ty = f.ty;
        println!("{:?}", field_name);
        let field_name_path = format_ident!("{}_path", field_name);
        let path_key = format_ident!("{}", field_name).to_string();

        quote! {
            pub fn #field_name_path() -> ::automerge::Path {
                ::automerge::Path::root().key(#path_key)
            }
        }
    });
    quote! {
        #[automatically_derived]
        impl #t_name {
            #(#imp_paths)*

            pub fn diff(&self, other: &Self) -> Vec<automerge::LocalChange> {
                todo!()
            }
        }
    }
}
