use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, GenericArgument, PathArguments, Type, parse_macro_input};

/// Injects `Arc<T>`, `Option<Arc<T>>`, and `Arc<dyn Trait>` fields.
/// Trait objects must be registered as `Arc<dyn Trait>` values.
#[proc_macro_derive(Injectable)]
pub fn injectable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand(input).unwrap_or_else(syn::Error::into_compile_error).into()
}

fn expand(mut input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let Data::Struct(data) = &input.data else {
        return Err(syn::Error::new_spanned(
            &input,
            "Injectable requires a struct with named Arc<T> fields",
        ));
    };
    let Fields::Named(fields) = &data.fields else {
        return Err(syn::Error::new_spanned(
            &input,
            "Injectable requires a struct with named Arc<T> fields",
        ));
    };
    let mut initializers = Vec::new();
    for field in &fields.named {
        let (dependency_type, optional) = match generic_inner(&field.ty, "Option") {
            Some(inner) => (inner, true),
            None => (&field.ty, false),
        };
        let dependency = generic_inner(dependency_type, "Arc").ok_or_else(|| {
            syn::Error::new_spanned(&field.ty, "Injectable fields must be Arc<T> or Option<Arc<T>>")
        })?;
        let (registered, resolve) = if matches!(dependency, Type::TraitObject(_)) {
            (
                quote! { ::std::sync::Arc<#dependency> },
                quote! {
                    ::std::sync::Arc::clone(instance.as_ref())
                },
            )
        } else {
            (quote! { #dependency }, quote! { instance })
        };
        let name = &field.ident;
        let value = if optional {
            quote! {
                match resolver.get::<#registered>() {
                    ::std::result::Result::Ok(instance) => ::std::option::Option::Some(#resolve),
                    ::std::result::Result::Err(::giong_di::Error::Missing(name))
                        if name == ::std::any::type_name::<#registered>() => ::std::option::Option::None,
                    ::std::result::Result::Err(error) => return ::std::result::Result::Err(error),
                }
            }
        } else {
            quote! {{ let instance = resolver.get::<#registered>()?; #resolve }}
        };
        initializers.push(quote! { #name: #value });
        input.generics.make_where_clause().predicates.push(syn::parse_quote! {
            #registered: ::std::any::Any + ::std::marker::Send + ::std::marker::Sync
        });
    }
    input.generics.make_where_clause().predicates.push(syn::parse_quote! {
        Self: ::std::any::Any + ::std::marker::Send + ::std::marker::Sync
    });
    let name = &input.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    Ok(quote! {
        impl #impl_generics ::giong_di::Injectable for #name #type_generics #where_clause {
            fn inject(resolver: &mut ::giong_di::Resolver<'_>) -> ::std::result::Result<Self, ::giong_di::Error> {
                ::std::result::Result::Ok(Self { #(#initializers),* })
            }
        }
    })
}

fn generic_inner<'a>(ty: &'a Type, name: &str) -> Option<&'a Type> {
    let Type::Path(path) = ty else { return None };
    if path.qself.is_some() {
        return None;
    }
    let segment = path.path.segments.last()?;
    if segment.ident != name {
        return None;
    }
    let PathArguments::AngleBracketed(arguments) = &segment.arguments else { return None };
    if arguments.args.len() != 1 {
        return None;
    }
    let GenericArgument::Type(inner) = arguments.args.first()? else { return None };
    Some(inner)
}
