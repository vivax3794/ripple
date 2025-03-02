extern crate proc_macro;
use proc_macro2::TokenStream;
use quote::format_ident;
use syn::ItemStruct;
use template_quote::{ToTokens, quote};

#[proc_macro_derive(Component)]
pub fn component_derive(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item = syn::parse_macro_input!(item as ItemStruct);
    let result = implementation(item);
    result.into()
}

fn implementation(item: ItemStruct) -> TokenStream {
    let name = item.ident.clone();
    let (fields, is_named) = get_fields(item.fields);

    let data_name = format_ident!("_{name}Data");

    quote! {
        #[doc(hidden)]
        #(if is_named) {
            pub struct #data_name {
                #(for field in &fields) {
                        pub #{field.access.clone()}: ::natrix::macro_ref::Signal<#{field.type_.clone()}, Self>,
                }
            }
        } #(else) {
            pub struct #data_name(
                #(for field in &fields) {
                        pub ::natrix::macro_ref::Signal<#{field.type_.clone()}, Self>,
                }
            );
        }

        impl ::natrix::macro_ref::ComponentData for #data_name {
            fn signals(&self) -> ::std::vec::Vec<&dyn ::natrix::macro_ref::SignalMethods<Self>> {
                ::std::vec![
                    #(for field in &fields) {
                        &self.#{field.access.clone()},
                    }
                ]
            }
            fn signals_mut(&mut self) -> ::std::vec::Vec<&mut dyn ::natrix::macro_ref::SignalMethods<Self>> {
                ::std::vec![
                    #(for field in &fields) {
                        &mut self.#{field.access.clone()},
                    }
                ]
            }
        }

        impl ::natrix::macro_ref::ComponentBase for #name {
            type Data = #data_name;
            fn into_data(self) -> Self::Data {
                #data_name {
                    #(for field in fields) {
                        #{field.access.clone()}: ::natrix::macro_ref::Signal::new(self.#{field.access}),
                    }
                }
            }
        }
    }
}

fn get_fields(fields: syn::Fields) -> (Vec<Field>, bool) {
    match fields {
        syn::Fields::Unit => (vec![], true),
        syn::Fields::Named(fields) => (
            fields
                .named
                .into_iter()
                .map(|field| Field {
                    type_: field.ty.into_token_stream(),
                    access: field.ident.into_token_stream(),
                })
                .collect(),
            true,
        ),
        syn::Fields::Unnamed(fields) => (
            fields
                .unnamed
                .into_iter()
                .enumerate()
                .map(|(index, field)| Field {
                    type_: field.ty.to_token_stream(),
                    access: proc_macro2::Literal::usize_unsuffixed(index).to_token_stream(),
                })
                .collect(),
            false,
        ),
    }
}

struct Field {
    type_: TokenStream,
    access: TokenStream,
}
