use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Expr, ItemFn};

#[proc_macro_attribute]
pub fn create_tool_with_function(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input function
    let input_fn = parse_macro_input!(item as ItemFn);

    // Parse the attribute as an expression
    let attr_expr = parse_macro_input!(attr as Expr);

    // Extract the function name
    let fn_name = &input_fn.sig.ident;

    // Create a variable name for the Tool instance
    let tool_var_name = format_ident!("{}_TOOL", fn_name.to_string().to_uppercase());

    // Extract argument names and types
    let inputs = &input_fn.sig.inputs;
    let mut arg_names = Vec::new();
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
            arg_type_tokens.push(arg_type.clone());
        }
    }

    // Generate the code to create the Tool structn
    let gen = quote! {
        #input_fn


        pub static #tool_var_name: Lazy<Tool> = Lazy::new(|| {
            let arg_names = vec![#(stringify!(#arg_names).to_string()),*];
            let arg_types = vec![#(stringify!(#arg_type_tokens).to_string()),*];

            let func = {
                use std::sync::Arc;
                let func = Arc::new(move |args: &[SupportedType]| -> MyResult<String> {
                    let parsers = get_parsers();

                    let mut iter = args.iter();
                    #(
                        let arg_type = stringify!(#arg_type_tokens);
                        let #arg_names = {
                            let arg = iter.next().ok_or("Not enough arguments")?.clone();
                            let parser = parsers.get(arg_type)
                                .ok_or(format!("Parser not found for type {}", arg_type))?;
                            let any_val = parser(arg)?;
                            let val = any_val.downcast::<#arg_type_tokens>()
                                .map_err(|_| "Type mismatch")?;
                            *val
                        };
                    )*

                    #fn_name(#(#arg_names),*)
                }) as Arc<dyn Fn(&[SupportedType]) -> MyResult<String> + Send + Sync>;
                func
            };

            let tool_def_obj = #attr_expr;

            // Ensure tool_def_obj is a &str
            let tool_def_obj_ref: &str = &tool_def_obj;

            Tool {
                name: serde_json::from_str::<serde_json::Value>(tool_def_obj_ref).unwrap()["name"]
                    .as_str()
                    .unwrap()
                    .to_string(),
                function: func,
                tool_def_obj: tool_def_obj.clone(),
                arg_names: arg_names,
                arg_types: arg_types,
            }
        });
    };

    gen.into()
}
