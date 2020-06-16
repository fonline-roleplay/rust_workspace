use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields, Visibility, Type, Ident};

fn convert_type(ty: &str) -> Option<&'static str> {
    Some(match ty {
        "u8" => "uint8",
        "u16" => "uint16",
        "u32" => "uint",
        "u64" => "uint64",
        "i8" => "int8",
        "i16" => "int16",
        "i32" => "int",
        "i64" => "int64",
        "bool" => "bool",
        "f32" => "float",
        "f64" => "double",
        _ => return None,
    })
}

fn detect_type(ty: &Type) -> &'static str {
    match ty {
        Type::Path(type_path) if type_path.qself.is_none() => {
            match type_path.path.get_ident().map(|ident| ident.to_string()) {
                Some(ident) => {
                    match convert_type(&ident) {
                        Some(as_type) => {
                            as_type
                        },
                        None => panic!("Type \"{}\" is not supported", ident),
                    }
                },
                None => panic!("Only single word type idents are supported"),
            }
        },
        _ => panic!("Only absolute path types are supported"),
    }
}

fn detect_fields(data: &Data) -> Vec<(&'static str, Ident)> {
    match data {
        Data::Struct(data_struct) => {
            match &data_struct.fields {
                Fields::Named(fields_named) => {
                    fields_named.named.iter().filter_map(|field| {
                        if let Visibility::Public(..) = field.vis {
                            let as_type = detect_type(&field.ty);
                            let name = field.ident.clone().expect("Field name");
                            Some((as_type, name))
                        } else {
                            None
                        }
                    }).collect()                    
                },
                _ => panic!("Only named fileds are supported"),
            }
        },
        _ => panic!("Only structs are supported"),
    }
}

#[proc_macro_derive(AngelScript)]
pub fn my_macro(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let name_str = name.to_string();
    let generics = input.generics;

    let fields = detect_fields(&input.data);
    let fragments: Vec<_> = fields.iter().map(|(ty, field)| {
        let full = format!("{} {}", ty, field.to_string());
        quote! {
            let offset = offset_of!(#name, #field);
            engine.register_object_property::<Self>(#name_str, #full, offset).expect("Register field for Rust struct in AS");
        }
    }).collect();

    // Build the output, possibly using quasi-quotation
    let expanded = quote! {
        impl #generics AngelScript for #name #generics {
            fn register<E: Engine>(engine: &mut E) {
                use memoffset::offset_of;
                engine.register_object_type::<Self>(#name_str, 0).expect("Register Rust struct in AS");
                #(
                    #fragments
                )*
            }
        }
    };

    // Hand the output tokens back to the compiler
    TokenStream::from(expanded)
}
