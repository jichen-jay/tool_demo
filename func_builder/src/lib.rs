use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, Expr};

#[proc_macro_attribute]
pub fn create_tool_with_function(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input_fn = parse_macro_input!(item as ItemFn);
    let attr_expr = parse_macro_input!(_attr as Expr);

    // Extract the function name
    let fn_name = &input_fn.sig.ident;

    // Extract argument names and types
    let inputs = &input_fn.sig.inputs;
    let mut arg_names = Vec::new();
    let mut arg_types = Vec::new();
    let mut arg_type_tokens = Vec::new();

    for arg in inputs {
        if let syn::FnArg::Typed(pat_type) = arg {
            // Get the argument name
            if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                let arg_name = &pat_ident.ident;
                arg_names.push(quote! { #arg_name });
            }
            // Get the argument type
            let arg_type = &*pat_type.ty;
            arg_types.push(quote! { #arg_type }.to_string());
            arg_type_tokens.push(arg_type.clone());
        }
    }

    // Generate the code to create the Tool struct
    let gen = quote! {
        #input_fn

        fn create_tool() -> Tool {
            let arg_names = vec![#(stringify!(#arg_names).to_string()),*];
            let arg_types = vec![#(stringify!(#arg_type_tokens)),*];

            let func = {
                use std::sync::Arc;
                let func = Arc::new(move |args: &[SupportedType]| -> MyResult<String> {
                    let parsers = get_parsers();

                    let mut iter = args.iter();
                    #(
                        let #arg_names = {
                            let arg = iter.next().ok_or("Not enough arguments")?.clone();
                            let parser = parsers.get(#arg_types).ok_or("Parser not found")?;
                            let any_val = parser(arg)?;
                            let val = any_val.downcast::<#arg_type_tokens>().map_err(|_| "Type mismatch")?;
                            *val
                        };
                    )*

                    #fn_name(#(#arg_names),*)
                }) as Arc<dyn Fn(&[SupportedType]) -> MyResult<String> + Send + Sync>;
                func
            };

            Tool {
                name: serde_json::from_str::<serde_json::Value>(#attr_expr).unwrap()["name"]
                    .as_str()
                    .unwrap()
                    .to_string(),
                function: func,
                tool_def_obj: #attr_expr,
                arg_names: vec![#(stringify!(#arg_names).to_string()),*],
                arg_types: vec![#(stringify!(#arg_type_tokens).to_string()),*],
            }
        }
    };

    gen.into()
}
