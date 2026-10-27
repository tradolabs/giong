use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Data, DeriveInput, Field, Fields, GenericArgument, PathArguments, Type, parse_macro_input,
};

#[proc_macro_derive(Injectable)]
pub fn injectable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand(input).unwrap_or_else(syn::Error::into_compile_error).into()
}

fn expand(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
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

    let initializers = fields.named.iter().map(expand_field).collect::<syn::Result<Vec<_>>>()?;
    let name = &input.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics ::giong_di::Injectable for #name #type_generics #where_clause {
            fn inject(
                resolver: &mut ::giong_di::Resolver<'_>,
            ) -> ::std::result::Result<Self, ::giong_di::Error> {
                ::std::result::Result::Ok(Self { #(#initializers),* })
            }
        }
    })
}

fn expand_field(field: &Field) -> syn::Result<proc_macro2::TokenStream> {
    let name = &field.ident;
    let (arc_type, is_optional) = match unwrap_type(&field.ty, "Option") {
        Some(inner) => (inner, true),
        None => (&field.ty, false),
    };

    let Some(target) = unwrap_type(arc_type, "Arc") else {
        return Err(syn::Error::new_spanned(
            &field.ty,
            "Injectable fields must be Arc<T> or Option<Arc<T>>",
        ));
    };

    let is_dyn = matches!(target, Type::TraitObject(_));
    let resolve = match (is_optional, is_dyn) {
        (false, false) => quote! { resolver.get::<#target>()? },
        (true, false) => quote! { resolver.get_optional::<#target>()? },
        (false, true) => quote! { resolver.get_dyn::<#target>()? },
        (true, true) => quote! { resolver.get_optional_dyn::<#target>()? },
    };

    Ok(quote! { #name: #resolve })
}

fn unwrap_type<'a>(ty: &'a Type, wrapper: &str) -> Option<&'a Type> {
    let Type::Path(path) = ty else { return None };
    let segment = path.path.segments.last()?;
    if segment.ident != wrapper {
        return None;
    }
    let PathArguments::AngleBracketed(args) = &segment.arguments else { return None };
    let GenericArgument::Type(inner) = args.args.first()? else { return None };
    Some(inner)
}
