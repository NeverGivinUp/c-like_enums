extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
extern crate syn;

use std::collections::HashSet;

use proc_macro2::Ident;
use proc_macro2::TokenStream;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(CLikeTryFrom)]
pub fn c_like_enum_from_derive_macro(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = parse_macro_input!(input as DeriveInput);

    // Build the trait implementation
    impl_c_like_enum_from(&ast)
}

// necessary, as there is no way (I know of) to compare enums by discriminator only,
// much less to have them hashed by the discriminator only.
fn lit_to_string(lit: &syn::Lit) -> String {
    match lit {
        syn::Lit::Str(_) => "string",
        syn::Lit::ByteStr(_) => "byte_str",
        syn::Lit::Byte(_) => "byte",
        syn::Lit::Char(_) => "char",
        syn::Lit::Int(_) => "int",
        syn::Lit::Float(_) => "float",
        syn::Lit::Bool(_) => "bool",
        syn::Lit::Verbatim(_) => "verbatim",
    }.to_string()
}


fn extract_variant_data<'a>(enum_variant: &'a syn::Variant, enum_name: &syn::Ident)
                            -> (Option<String>, Option<&'a str>, proc_macro2::TokenStream) {
    let variant_name = &enum_variant.ident;
    if let syn::Fields::Unnamed(ref fields) = enum_variant.fields {
        if !&fields.unnamed.is_empty() {
            panic!("variant {} of enum {} has fields!", variant_name, enum_name)
        };
    };

    if let Some((_, ref expr)) = &enum_variant.discriminant {
        if let syn::Expr::Lit(lit) = expr {
            // for pure comparison std::mem::discriminant could be used as well
            let discr_type = Some(lit_to_string(&lit.lit));
            match &lit.lit {
                syn::Lit::Int(ref int_lit) => {
                    let suffix = int_lit.suffix();
                    let int_type = Some(suffix);
                    let variant_val = int_lit;
                    (discr_type, int_type, quote!(#variant_val => Ok(#enum_name::#variant_name)))
                }
                _ => panic!(format!("not implemented for discriminant type {}", discr_type.unwrap()))
            }
        } else {
            panic!(format!("c-like enums' discriminant fields must be literals, but {} is not!", enum_variant.ident));
        }
    } else {
        panic!(format!("c-like enums must have discriminant fields, but {} does not have one!", enum_variant.ident));
    }
}


fn extract_repr_type(ast: &syn::DeriveInput) -> Option<String> {
    if let Some(attribute_tts) = &ast.attrs.iter()
        .filter(|attr| {
            // println!("processing attribute {:?}", attr.path);
            if syn::AttrStyle::Outer == attr.style {
                if let Some(pair) = attr.path.segments.first() {
                    let seg: &syn::PathSegment = pair.into();
                    let matching = "repr".to_string() == seg.ident.to_string();
                    //       println!("segment ident is matching expected_ident: '{}'", matching);
                    matching
                } else { true }
            } else { true }
        })
        .map(|attr| {
            //println!("retained attribute {:?} ", attr.path);
            &attr.tokens
        })
        .next() {
        if let Some(proc_macro2::TokenTree::Group(g)) = (**attribute_tts).clone().into_iter().next() {
            if let Some(proc_macro2::TokenTree::Ident(i)) = g.stream().into_iter().next() {
                Some(i.to_string())
            } else {
                panic!(format!("Tokenstream g: {:#?} is not an Ident", g))
            }
        } else { panic!(format!("Tokenstream attribute_tts: {:#?} is not an Ident", attribute_tts)) }
    } else {
        None
    }
}

fn impl_c_like_enum_from(ast: &syn::DeriveInput) -> proc_macro::TokenStream {
    let enum_name = &ast.ident;
//    println!("expanding macro c-like enum for {}", enum_name);

    let repr_type = extract_repr_type(&ast)
        .expect(&format!("Size of c-like enum '{}' must be defined with \
    an attribute #[repr(X)], where X any integer type but is not 'usize' or 'isize', as those \
    are the compile-target dependent pointer sizes \
    https://doc.rust-lang.org/std/primitive.usize.html", enum_name)[..]);


    let mut types: HashSet<Option<String>> = HashSet::new();
    let mut sub_types: HashSet<Option<&str>> = HashSet::new();
    let variants: Vec<proc_macro2::TokenStream> = if let syn::Data::Enum(enum_data) = &ast.data {
        if enum_data.variants.is_empty() { return quote!().into(); };
        enum_data.variants.iter().map(|variant| {
            let (discr_type, int_type, variant_mapping)
                = extract_variant_data(variant, enum_name);
            types.insert(discr_type);
            sub_types.insert(int_type);
            variant_mapping
        }).collect()
    } else {
        panic!(format!("Can only derive enums, but {} is not one", &ast.ident));
    };


    if types.len() > 1 {
        panic!(format!("c-like enums' discriminant literals must all be of the same type, but \
        variants of {} have {:?}!", enum_name, types));
    }
    if sub_types.len() > 1 {
        panic!(format!("c-like enums' discriminant literals must all be of the same type, but \
        variants of {}  have {:?}!", enum_name, sub_types));
    }


    if !variants.is_empty()
        && (types.is_empty() || *types.iter().next().unwrap() == None
        || sub_types.is_empty() || *sub_types.iter().next().unwrap() == None) {
        panic!("enum has variants, but no discriminant types.");
    }
    let discriminant_type = types.into_iter().next().unwrap().unwrap();
    let sub_discr_type = sub_types.into_iter().next().unwrap().unwrap();

    let from_type = if discriminant_type == "int".to_string() {
        let int_type = sub_discr_type.to_lowercase();
        if repr_type != int_type {
            panic!(format!("Type of discriminants '{}' does not match attribute #[repr({})]", int_type, repr_type));
        }

        match sub_discr_type {
            "" => to_token_stream("usize"),
            _ => to_token_stream(sub_discr_type)
        }
    } else {
        panic!("somethings wrong in this macro")
    };


    expand(from_type, enum_name, variants).into()
//panic!(expand(from_type, enum_name, variants).to_string());
}

fn expand(from_type: TokenStream, enum_name: &Ident, variants: Vec<proc_macro2::TokenStream>) -> TokenStream {
    quote! {
        impl TryFrom <#from_type> for #enum_name {
            type Error = TryFromIntError<#from_type>;
            fn try_from (num: #from_type) -> Result<Self, TryFromIntError<#from_type>> {
                match num{
                    #(#variants),*,
                    _=> Err(TryFromIntError(num))
                }
            }
        }
    }
}

fn to_token_stream(int_suffix: &str) -> proc_macro2::TokenStream {
    proc_macro2::TokenStream::from(
        proc_macro::TokenStream::from(
            proc_macro::TokenTree::Ident(
                proc_macro::Ident::new(
                    &int_suffix.to_lowercase()[..],
                    proc_macro::Span::call_site()))))
}

