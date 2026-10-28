use std::fs::read;

use proc_macro2::{Literal, TokenStream};
use quote::{ToTokens, quote};
use syn::{
    Data, DataStruct, DeriveInput, Expr, ExprLit, ExprTuple, Fields, Type, punctuated::Punctuated,
    token::Comma,
};

#[proc_macro_derive(Parsable)]
pub fn parsable_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse(input).unwrap();

    impl_parsable_derive(ast).into()
}
#[proc_macro_derive(ParsableAction, attributes(name))]
pub fn parsable_action_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse(input).unwrap();

    impl_parsable_action_derive(ast).into()
}

fn create_parser(ty: Type) -> TokenStream {
    quote! {
        <#ty as parser::Parsable>::parse(input)?
    }
}

fn create_fields(fields: Fields) -> TokenStream {
    match fields {
        Fields::Named(f) => {
            let fields = f.named.into_iter().map(|field| {
                let ident = field.ident.as_ref().unwrap();
                let parser = create_parser(field.ty);

                quote! {
                    #ident: #parser
                }
            });

            quote! {
                {
                    #(
                        #fields,
                    )*
                }
            }
        }
        Fields::Unit => quote! {},
        Fields::Unnamed(f) => {
            let fields = f.unnamed.into_iter().map(|field| create_parser(field.ty));

            quote! {
                (
                    #(
                        #fields
                    ),*
                )
            }
        }
    }
}

fn impl_parsable_derive(input: DeriveInput) -> TokenStream {
    let name = &input.ident;
    let fields: TokenStream = match input.data {
        Data::Struct(s) => {
            let fields = create_fields(s.fields);
            quote! {
                Some(Self
                    #fields
                )
            }
        }
        Data::Enum(e) => {
            let variants = e.variants.into_iter().map(|variant| {
                let name = variant.ident;
                let name_lower = name.to_string().to_lowercase();
                if !matches!(variant.fields, Fields::Unit) {
                    panic!("Unsupported type");
                }

                quote! {
                    #name_lower => Some(Self::#name)
                }
            });

            quote! {
                match input.next()? {
                    #(
                        #variants,
                    )*
                    _ => None,
                }
            }
        }
        _ => panic!("Unsupported Type"),
    };

    quote! {
        #[automatically_derived]
        impl Parsable for #name {
            fn parse<'a>(input: &mut impl Iterator<Item = &'a str>) -> Option<Self> {
                #fields
            }
        }
    }
}

fn get_next_string(elems: &mut Punctuated<Expr, Comma>) -> String {
    match elems.pop().unwrap() {
        Expr::Lit(ExprLit {
            lit: syn::Lit::Str(s),
            ..
        }) => s.value(),
        _ => panic!("Unsupported type inside tuple"),
    }
}

fn impl_parsable_action_derive(input: DeriveInput) -> TokenStream {
    let ident = input.ident;

    let mut name = ident.to_string().to_lowercase();

    // If it's a empty struct take just "name" as input instead of  "name()"
    if matches!(
        input.data,
        Data::Struct(DataStruct {
            fields: Fields::Unit,
            ..
        })
    ) {
        return quote! {
            #[automatically_derived]
            impl ParsableAction for #ident {
              fn parse_action(input: &str, actions: &mut Vec<Box<dyn Action>>) -> bool {
                  if input == #name {
                    actions.push(Box::new(Self));
                    true
                  } else {
                    false
                  }
              }
            }
        };
    }

    let mut start = "(".to_string();
    let mut end = ")".to_string();

    for attr in input.attrs {
        if attr.path().is_ident("name") {
            let mut expr: Expr = attr.parse_args().unwrap();
            match expr {
                Expr::Lit(ExprLit {
                    lit: syn::Lit::Str(s),
                    ..
                }) => {
                    name = s.value();
                }
                Expr::Tuple(ExprTuple { ref mut elems, .. }) => match elems.len() {
                    1 => {
                        name = get_next_string(elems);
                    }
                    2 => {
                        end = get_next_string(elems);
                        start = get_next_string(elems);
                        name = String::new();
                    }
                    3 => {
                        end = get_next_string(elems);
                        start = get_next_string(elems);
                        name = get_next_string(elems);
                    }
                    _ => panic!("Unsupported amount of elements in tuple"),
                },
                _ => panic!("Unsupported experssion inside name attribute"),
            }
        }
    }

    let prefix = format!("{name}{start}");

    quote! {
        #[automatically_derived]
        impl ParsableAction for #ident {
          fn parse_action(input: &str, actions: &mut Vec<Box<dyn Action>>) -> bool {
              if let Some(mut input) = input.strip_prefix(#prefix).and_then(|s| s.strip_suffix(#end)).map(|s| s.split(","))
              && let Some(action) = <#ident as parser::Parsable>::parse(&mut input) {
                  actions.push(Box::new(action));
                true
              } else {
                false
              }
          }
        }
    }
}
