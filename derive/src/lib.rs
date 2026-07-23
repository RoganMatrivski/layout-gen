use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, GenericArgument, PathArguments, Type, parse_macro_input};

#[proc_macro_derive(FromXmlAttrs)]
pub fn derive_from_xml_attrs(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;

    let fields = match &ast.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(named) => &named.named,
            _ => panic!("FromXmlAttrs only supports structs with named fields"),
        },
        _ => panic!("FromXmlAttrs only supports structs, not enums or unions"),
    };

    let parses = fields.iter().map(|f| {
        let ident = f.ident.as_ref().expect("named field");
        let attr_name = ident.to_string().replace('_', "-");
        let ty = &f.ty;

        if let Some(inner_ty) = option_inner_type(ty) {
            // Option<T> fields: attr present -> Some(parsed), attr absent -> defaults value.
            // A *present but malformed* attribute is still a hard error, not a silent None.
            quote! {
                let #ident: #ty = match node.attribute(#attr_name) {
                    Some(s) => Some(s.parse::<#inner_ty>().map_err(|e| {
                        eyre::eyre!("failed to parse `{}`: {}", #attr_name, e)
                    })?),
                    None => defaults.#ident.clone(),
                };
            }
        } else {
            quote! {
                let #ident: #ty = match node.attribute(#attr_name) {
                    Some(s) => s.parse::<#ty>().map_err(|e| {
                        eyre::eyre!("failed to parse `{}`: {}", #attr_name, e)
                    })?,
                    None => defaults.#ident.clone(),
                };
            }
        }
    });

    let field_names = fields.iter().map(|f| f.ident.as_ref().unwrap());

    let expanded = quote! {
        impl crate::commons::FromXmlAttrs for #name {
            type Error = eyre::Error;

            fn from_node(node: roxmltree::Node, defaults: &Self) -> eyre::Result<Self> {
                #(#parses)*

                Ok(Self { #(#field_names),* })
            }
        }
    };

    expanded.into()
}

/// If `ty` is `Option<T>`, returns `Some(&T)`. Otherwise `None`.
fn option_inner_type(ty: &Type) -> Option<&Type> {
    let Type::Path(type_path) = ty else {
        return None;
    };
    let segment = type_path.path.segments.last()?;
    if segment.ident != "Option" {
        return None;
    }
    let PathArguments::AngleBracketed(args) = &segment.arguments else {
        return None;
    };
    match args.args.first()? {
        GenericArgument::Type(inner) => Some(inner),
        _ => None,
    }
}
