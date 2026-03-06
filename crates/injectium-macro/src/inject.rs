use quote::quote;
use syn::{Data, DeriveInput, Error, Fields, Ident, Result, Type};

pub(crate) fn expand_inject(input: DeriveInput) -> Result<proc_macro2::TokenStream> {
    let (impl_gens, ty_gens, where_clause) = input.generics.split_for_impl();
    let (idents, types) = parse_struct_fields(&input)?;

    let name = &input.ident;

    Ok(quote! {
        impl #impl_gens injectium::Injectable for #name #ty_gens #where_clause {
            fn from_container(container: &injectium::Container) -> Self {
                Self {
                    #(#idents: container.get::<#types>(),)*
                }
            }

            fn try_from_container(container: &injectium::Container) -> Option<Self> {
                Some(Self {
                    #(#idents: container.try_get::<#types>()?,)*
                })
            }
        }

        // Register each field type as a declared dependency so that
        // `Container::validate` can verify registration at startup.
        #(
            injectium::declare_dependency!(#types);
        )*
    })
}

fn parse_struct_fields(input: &DeriveInput) -> Result<(Vec<Ident>, Vec<Type>)> {
    let fields = match &input.data {
        Data::Struct(s) => &s.fields,
        _ => {
            return Err(Error::new_spanned(
                input,
                "#[derive(Injectable)] only supports named structs",
            ));
        }
    };

    match fields {
        Fields::Named(n) => {
            let idents: Vec<_> = n
                .named
                .iter()
                .map(|f| f.ident.as_ref().unwrap().clone())
                .collect();
            let types: Vec<_> = n.named.iter().map(|f| f.ty.clone()).collect();

            Ok((idents, types))
        }
        _ => Err(Error::new_spanned(
            input,
            "#[derive(Injectable)] only supports named structs",
        )),
    }
}
