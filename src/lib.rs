// use proc_macro::TokenStream;
// use quote::{quote, format_ident};
// use syn::{parse_macro_input, ItemFn, FnArg, Type};

// #[proc_macro_attribute]
// pub fn register_tool(_attr: TokenStream, item: TokenStream) -> TokenStream {
//     let input = parse_macro_input!(item as ItemFn);
//     let fn_name = &input.sig.ident;
//     let fn_args = &input.sig.inputs;

//     let mut json_properties = quote!();
//     let mut arg_extractions = quote!();
//     let mut fn_call_args = quote!();

//     for arg in fn_args {
//         if let FnArg::Typed(pat_type) = arg {
//             if let Type::Reference(type_reference) = &*pat_type.ty {
//                 if let Type::Path(type_path) = &*type_reference.elem {
//                     let arg_name = &pat_type.pat;
//                     let arg_type = &type_path.path.segments.last().unwrap().ident;
                    
//                     json_properties.extend(quote! {
//                         #arg_name: {
//                             "type": #arg_type.to_lowercase(),
//                             "description": stringify!(#arg_name)
//                         },
//                     });

//                     arg_extractions.extend(quote! {
//                         let #arg_name = args.get(stringify!(#arg_name))
//                             .ok_or_else(|| anyhow::anyhow!(concat!("Missing ", stringify!(#arg_name))))?;
//                     });

//                     fn_call_args.extend(quote! {
//                         #arg_name,
//                     });
//                 }
//             }
//         }
//     }

//     let expanded = quote! {
//         #input

//         inventory::submit! {
//             Tool::new(
//                 stringify!(#fn_name).to_string(),
//                 serde_json::json!({
//                     "name": stringify!(#fn_name),
//                     "description": concat!("Automatically generated description for ", stringify!(#fn_name)),
//                     "parameters": {
//                         "type": "object",
//                         "properties": {
//                             #json_properties
//                         },
//                         "required": [#(stringify!(#fn_args)),*]
//                     }
//                 }),
//                 |args: &std::collections::HashMap<String, String>| {
//                     #arg_extractions
//                     #fn_name(#fn_call_args)
//                 }
//             )
//         }

//         inventory::collect!(Tool);
//     };

//     TokenStream::from(expanded)
// }
// use anyhow::Result;
// use serde_json::Value;
// use std::collections::HashMap;

// // This would be your preprocessor function
// fn register_tool(json_description: &str, func: fn(&str, &str) -> Result<String>) -> Tool {
//     let description: Value = serde_json::from_str(json_description).unwrap();
    
//     Tool {
//         name: description["name"].as_str().unwrap().to_string(),
//         description: description["description"].as_str().unwrap().to_string(),
//         parameters: description["parameters"].clone(),
//         function: Box::new(move |args: &HashMap<String, String>| {
//             let location = args.get("location").ok_or_else(|| anyhow::anyhow!("Missing location"))?;
//             let unit = args.get("unit").ok_or_else(|| anyhow::anyhow!("Missing unit"))?;
//             func(location, unit)
//         }),
//     }
// }

// struct Tool {
//     name: String,
//     description: String,
//     parameters: Value,
//     function: Box<dyn Fn(&HashMap<String, String>) -> Result<String>>,
// }

// impl Tool {
//     fn call(&self, args: &HashMap<String, String>) -> Result<String> {
//         (self.function)(args)
//     }
// }

// // Your original function
// fn get_current_weather(location: &str, unit: &str) -> Result<String> {
//     Ok(format!("fake_weather for {} in {}", location, unit))
// }

// fn main() -> Result<()> {
//     // JSON description of the function
//     let weather_tool_json = r#"
//     {
//         "name": "get_current_weather",
//         "description": "Get the current weather in a given location",
//         "parameters": {
//             "type": "object",
//             "properties": {
//                 "location": {
//                     "type": "string",
//                     "description": "The city and state, e.g. San Francisco, CA"
//                 },
//                 "unit": {
//                     "type": "string",
//                     "enum": ["celsius", "fahrenheit"],
//                     "description": "The unit of measurement"
//                 }
//             },
//             "required": ["location", "unit"]
//         }
//     }
//     "#;

//     // Register the tool
//     let weather_tool = register_tool(weather_tool_json, get_current_weather);

//     // Use the tool
//     let mut args = HashMap::new();
//     args.insert("location".to_string(), "San Francisco, CA".to_string());
//     args.insert("unit".to_string(), "fahrenheit".to_string());
    
//     let result = weather_tool.call(&args)?;
//     println!("Result: {}", result);

//     Ok(())
// }
