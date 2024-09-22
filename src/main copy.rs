use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

struct Tool {
    name: String,
    json_description: String,
    function: Box<dyn Fn(&[&str]) -> Result<String> + Send + Sync>,
    arg_names: Vec<String>,
    arg_types: Vec<String>,
}

//intend to use this macro to extract the arg names and arg types into 2 array
//intend to use it like a function, returning (Vec<String>, Vec<String>)
macro_rules! extract_args_and_types {
    (
        fn $func_name:ident($($arg_name:ident : $arg_type:ty),*) -> $ret_type:ty
    ) => {
        const ARG_NAMES: &[&str] = &[$(stringify!($arg_name)),*];
        const ARG_TYPES: &[&str] = &[$(stringify!($arg_type)),*];
    };
}

//intend to use this macro to convert
// fn process_values(a: i32, b: f32, c: bool, d: &str, e: i32) -> String {
//     format!(
//         "Processed: a = {}, b = {}, c = {}, d = {}, e = {}",
//         a, b, c, d, e
//     )
// }
// to Fn(&[&str]) -> Result<String> type, like
// fn process_values_intermediary(&[&str]) -> String {
//     format!(
//         "Processed: a = {}, b = {}, c = {}, d = {}, e = {}",
//         a, b, c, d, e
//     )
// }
#[macro_export]
macro_rules! convert_function {
    ($func:ident, $arg_names:expr) => {{
        let arg_names = $arg_names.clone(); // Capture the argument names in the closure
        Box::new(move |args: &[&str]| -> Result<String> {
            let mut iter = args.iter();
            let mut extracted_args = Vec::new();
            for arg_name in &arg_names {
                let arg = iter
                    .next()
                    .ok_or_else(|| anyhow!("Missing argument: {}", arg_name))?;
                extracted_args.push(arg.to_string());
            }
            $func(&extracted_args[0], &extracted_args[1])
        }) as Box<dyn Fn(&[&str]) -> Result<String> + Send + Sync>
    }};
}

impl Tool {
    fn call(&self, arguments_w_val: Value) -> Result<String> {
        let arguments = arguments_w_val["arguments"].as_array().unwrap();

        let arg_map = arguments
            .into_iter()
            .map(|arg| arg.as_object().unwrap().iter().next().unwrap())
            .map(|(k, v)| (k.clone(), v.as_str().unwrap().to_string()))
            .collect::<HashMap<String, String>>();

        let mut ordered_vals = Vec::new();
        for (arg_name, arg_type) in self.arg_names.iter().zip(self.arg_types.iter()) {
            let arg_value = arg_map.get(arg_name).unwrap();
            let converted_val = convert_value(arg_type, arg_value)?;
            ordered_vals.push(converted_val);
        }

        let args: Vec<&str> = ordered_vals.iter().map(|s| s.as_str()).collect();

        (self.function)(&args)
    }
}

fn convert_value(arg_type: &str, value: &str) -> Result<String> {
    match arg_type {
        "i32" => value
            .parse::<i32>()
            .map(|v| v.to_string())
            .map_err(|e| e.into()),
        "f32" => value
            .parse::<f32>()
            .map(|v| v.to_string())
            .map_err(|e| e.into()),
        "bool" => value
            .parse::<bool>()
            .map(|v| v.to_string())
            .map_err(|e| e.into()),
        "str" => Ok(value.to_string()),
        _ => Err(format!("Unsupported argument type: {}", arg_type).into()),
    }
}

// Example function to be wrapped
fn process_values(a: i32, b: f32, c: bool, d: &str, e: i32) -> String {
    format!(
        "Processed: a = {}, b = {}, c = {}, d = {}, e = {}",
        a, b, c, d, e
    )
}
fn create_tool<F>(
    json_description: &str,
    func: F,
    arg_names: Vec<String>,
    arg_types: Vec<String>,
) -> Tool
where
    F: Fn(&[&str]) -> Result<&str> + Send + Sync + 'static,
{
    let description: Value = serde_json::from_str(json_description).unwrap();

    Tool {
        name: description["name"].as_str().unwrap().to_string(),
        json_description: String,
        arg_names: arg_names,
        arg_types: arg_types,
        function: Box::new(func),
    }
}

// Function to wrap `process_values` into a `Tool`
// need to update the function so that it can create function automatically from the original function signature
// fn process_values(a: i32, b: f32, c: bool, d: &str, e: i32) -> String
fn the_wrapper_func(json_description: &str, arguments_w_val: Value) -> Tool {
    let function = Box::new(|args: &[&str]| -> Result<String> {
        let arguments = arguments_w_val["arguments"].as_array().unwrap();

        let arg_map = arguments
            .into_iter()
            .map(|arg| arg.as_object().unwrap().iter().next().unwrap())
            .map(|(k, v)| (k.clone(), v.as_str().unwrap().to_string()))
            .collect::<HashMap<String, String>>();

        let mut ordered_vals = Vec::new();
        for (arg_name, arg_type) in self.arg_names.iter().zip(self.arg_types.iter()) {
            let arg_value = arg_map.get(arg_name).unwrap();
            let converted_val = convert_value(arg_type, arg_value)?;
            ordered_vals.push(converted_val);
        }

        let args: Vec<&str> = ordered_vals.iter().map(|s| s.as_str()).collect();

        // the code to wrap in the original function, to invoke it
        Ok(process_values(a, b, c, d, e))
    });

    let (arg_names, arg_types) = extract_args_and_types!(fn process_values(a: i32, b: f32, c: bool, d: &str, e: i32) -> String);

    let tool: Tool = create_tool(json_description, func, arg_names, arg_types);

    tool
}

fn main() -> Result<()> {
    let json_description = r#"
    {
        "name": "process_values",
        "description": "Processes up to 5 different types of values",
        "parameters": {
            "type": "object",
            "properties": {
                "a": {
                    "type": "i32",
                    "description": "An integer value"
                },
                "b": {
                    "type": "f32",
                    "description": "A floating-point value"
                },
                "c": {
                    "type": "bool",
                    "description": "A boolean value"
                },
                "d": {
                    "type": "String",
                    "description": "A string value"
                },
                "e": {
                    "type": "i32",
                    "description": "Another integer value"
                }
            },
            "required": ["a", "b", "c", "d", "e"]
        }
    }
    "#;

    // Example JSON input
    let json_input = serde_json::json!({
        "arguments": [
            { "a": "42" },
            { "b": "3.14" },
            { "c": "true" },
            { "d": "example" },
            { "e": "100" }
        ]
    }); // Create a `Tool` that wraps the `process_values` function
    let tool = the_wrapper_func(&json_description, json_input);

    // Call the tool with the JSON input
    let result = tool.call(json_input)?;
    println!("{}", result); // Output: Processed: a = 42, b = 3.14, c = true, d = example, e = 100

    Ok(())
}
