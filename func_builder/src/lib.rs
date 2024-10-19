extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, FnArg, ItemFn, Pat, PatIdent, PatType};

#[proc_macro_attribute]
pub fn create_tool_with_function(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the function definition
    let input_fn = parse_macro_input!(item as ItemFn);

    // Extract the function name
    let func_name = &input_fn.sig.ident;

    // Extract function arguments
    let mut arg_names = Vec::new();
    let mut arg_types = Vec::new();
    let mut arg_idents = Vec::new();

    for input in &input_fn.sig.inputs {
        if let FnArg::Typed(PatType { pat, ty, .. }) = input {
            // Get the argument name and identifier
            if let Pat::Ident(PatIdent { ident, .. }) = &**pat {
                arg_names.push(ident.to_string());
                arg_idents.push(ident.clone());
            }
            // Get the argument type
            arg_types.push(ty.clone());
        }
    }

    let func_tool_name_str = format!("{}_tool", func_name);
    let func_tool_name = syn::Ident::new(&func_tool_name_str, func_name.span());

    // Generate the code
    let expanded = quote! {
        // Original function definition
        #input_fn

        // Function to get the Tool instance
        pub fn #func_tool_name() -> Tool {
            use std::sync::Arc;
            use std::error::Error;

            let arg_names = vec![#(stringify!(#arg_idents).to_string()),*];
            let arg_types = vec![#(stringify!(#arg_types).to_string()),*];

            let function = Arc::new(move |args: &[SupportedType]| -> Result<String, Box<dyn Error>> {
                let parsers = get_parsers();
                let mut iter = args.iter();

                // For storing temporary Strings to ensure they live long enough
                let mut temp_strings: Vec<String> = Vec::new();

                // Extract arguments
                #(
                    let #arg_idents: #arg_types = {
                        let arg = iter.next().ok_or("Not enough arguments")?;
                        let arg_type_str = stringify!(#arg_types);
                        let parser = parsers.get(arg_type_str)
                            .ok_or_else(|| format!("Parser not found for type {}", arg_type_str))?;
                        let boxed_value = parser(arg)?;

                        // Handle the downcasting based on the expected type
                        if arg_type_str == "&str" {
                            let string_value = boxed_value.downcast_ref::<String>()
                                .ok_or("Type mismatch when casting to String")?.clone();
                            temp_strings.push(string_value);
                            temp_strings.last().unwrap().as_str()
                        } else if arg_type_str == "String" {
                            boxed_value.downcast_ref::<String>()
                                .ok_or("Type mismatch when casting to String")?.clone()
                        } else if arg_type_str == "i32" {
                            *boxed_value.downcast_ref::<i32>()
                                .ok_or("Type mismatch when casting to i32")?
                        } else if arg_type_str == "f32" {
                            *boxed_value.downcast_ref::<f32>()
                                .ok_or("Type mismatch when casting to f32")?
                        } else if arg_type_str == "bool" {
                            *boxed_value.downcast_ref::<bool>()
                                .ok_or("Type mismatch when casting to bool")?
                        } else {
                            return Err(format!("Unsupported type: {}", arg_type_str).into());
                        }
                    };
                )*

                #func_name(#(#arg_idents),*).map_err(|e| e.into())
            }) as Arc<dyn Fn(&[SupportedType]) -> Result<String, Box<dyn Error>> + Send + Sync>;

            Tool {
                name: stringify!(#func_tool_name).to_string(),
                function,
                arg_names,
                arg_types,
            }
        }
    };

    TokenStream::from(expanded)
}
