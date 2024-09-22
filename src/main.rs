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
#[macro_export]
macro_rules! create_tool_with_function {
    (
        fn $func_name:ident($($arg_name:ident : $arg_type:ty),*) -> $ret_type:ty,
        $json_description:expr
    ) => {{
        let arg_names = vec![$(stringify!($arg_name).to_string()),*];
        let arg_types = vec![$(stringify!($arg_type).to_string()),*];

        let arg_names_cloned = arg_names.clone();
        let arg_types_cloned = arg_types.clone();

        let func = Box::new(move |args: &[&str]| -> Result<String> {
            let mut iter = args.iter();
            let mut parsed_args: Vec<Box<dyn std::any::Any>> = Vec::new();

            for (arg_name, arg_type) in arg_names_cloned.iter().zip(arg_types_cloned.iter()) {
                let arg = iter
                    .next()
                    .ok_or_else(|| format!("Missing argument: {}", arg_name))?;

                match arg_type.as_str() {
                    "i32" => parsed_args.push(Box::new(arg.parse::<i32>()?)),
                    "f32" => parsed_args.push(Box::new(arg.parse::<f32>()?)),
                    "bool" => parsed_args.push(Box::new(arg.parse::<bool>()?)),
                    "&str" | "String" => parsed_args.push(Box::new(arg.to_string())),
                    _ => return Err(format!("Unsupported argument type: {}", arg_type).into()),
                }
            }

            let result = $func_name(
                *parsed_args[0].downcast_ref::<i32>().unwrap(),
                *parsed_args[1].downcast_ref::<f32>().unwrap(),
                *parsed_args[2].downcast_ref::<bool>().unwrap(),
                parsed_args[3].downcast_ref::<String>().unwrap(),
                *parsed_args[4].downcast_ref::<i32>().unwrap(),
            );
            Ok(result)
        }) as Box<dyn Fn(&[&str]) -> Result<String> + Send + Sync>;

        Tool {
            name: serde_json::from_str::<Value>($json_description).unwrap()["name"]
                .as_str()
                .unwrap()
                .to_string(),
            json_description: $json_description.to_string(),
            arg_names,
            arg_types,
            function: func,
        }
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
        "&str" | "String" => Ok(value.to_string()),
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
    });

    // Use the combined macro to create the Tool
    let tool = create_tool_with_function!(
        fn process_values(a: i32, b: f32, c: bool, d: &str, e: i32) -> String,
        json_description
    );

    let result = tool.call(json_input)?;
    println!("{}", result); // Output: Processed: a = 42, b = 3.14, c = true, d = example, e = 100

    Ok(())
}
