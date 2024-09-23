use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Debug)]
enum SupportedType {
    I32(i32),
    F32(f32),
    Bool(bool),
    String(String),
}

// Original parse_argument function
fn parse_argument(arg_type: &str, arg_value: &str) -> Result<SupportedType> {
    match arg_type {
        "i32" => {
            let parsed_value = arg_value.parse::<i32>()?;
            Ok(SupportedType::I32(parsed_value))
        }
        "f32" => {
            let parsed_value = arg_value.parse::<f32>()?;
            Ok(SupportedType::F32(parsed_value))
        }
        "bool" => match arg_value {
            "true" => Ok(SupportedType::Bool(true)),
            "false" => Ok(SupportedType::Bool(false)),
            _ => Err(format!("Invalid boolean value: {}", arg_value).into()),
        },
        "String" => Ok(SupportedType::String(arg_value.to_string())),
        _ => Err(format!("Invalid type: {}", arg_type).into()),
    }
}

// Type-specific parse functions
fn parse_i32(arg_value: &str) -> i32 {
    match parse_argument("i32", arg_value) {
        Ok(SupportedType::I32(val)) => val,
        _ => panic!("Expected i32 for argument"),
    }
}

fn parse_f32(arg_value: &str) -> f32 {
    match parse_argument("f32", arg_value) {
        Ok(SupportedType::F32(val)) => val,
        _ => panic!("Expected f32 for argument"),
    }
}

fn parse_bool(arg_value: &str) -> bool {
    match parse_argument("bool", arg_value) {
        Ok(SupportedType::Bool(val)) => val,
        _ => panic!("Expected bool for argument"),
    }
}

fn parse_string(arg_value: &str) -> String {
    match parse_argument("String", arg_value) {
        Ok(SupportedType::String(val)) => val,
        _ => panic!("Expected String for argument"),
    }
}

// Tool struct that holds the function and argument metadata
struct Tool {
    name: String,
    function: Box<dyn Fn(&[&str]) -> Result<String> + Send + Sync>,
    arg_names: Vec<String>,
    arg_types: Vec<String>,
}

impl Tool {
    fn call(&self, arguments_w_val: Value) -> Result<String> {
        let arguments = arguments_w_val["arguments"].as_array().ok_or("Invalid arguments format")?;
        println!("Arguments: {:?}", arguments);
        println!("Argument names: {:?}", self.arg_names);
        println!("Argument types: {:?}", self.arg_types);

        let mut ordered_vals = Vec::new();

        for arg_name in &self.arg_names {
            let arg_value = arguments.iter().find_map(|arg| {
                let obj = arg.as_object().unwrap();
                obj.get(arg_name)
            });

            if let Some(arg_value) = arg_value {
                println!("Argument name: {}, Value: {:?}", arg_name, arg_value);
                ordered_vals.push(arg_value.as_str().ok_or("Invalid argument value")?);
            } else {
                return Err(format!("Missing argument: {}", arg_name).into());
            }
        }
        println!("Ordered values: {:?}", ordered_vals);

        (self.function)(&ordered_vals)
    }
}

// Macro to create a Tool with a function and argument parsing logic
#[macro_export]
macro_rules! create_tool_with_function {
    (
        fn $func_name:ident($arg1_name:ident : $arg1_type:ty, $arg2_name:ident : $arg2_type:ty, $arg3_name:ident : $arg3_type:ty, $arg4_name:ident : $arg4_type:ty, $arg5_name:ident : $arg5_type:ty) -> $ret_type:ty,
        $json_description:expr
    ) => {{
        let arg_names = vec![
            stringify!($arg1_name).to_string(),
            stringify!($arg2_name).to_string(),
            stringify!($arg3_name).to_string(),
            stringify!($arg4_name).to_string(),
            stringify!($arg5_name).to_string(),
        ];
        let arg_types = vec![
            stringify!($arg1_type).to_string(),
            stringify!($arg2_type).to_string(),
            stringify!($arg3_type).to_string(),
            stringify!($arg4_type).to_string(),
            stringify!($arg5_type).to_string(),
        ];

        let arg_types_clone = arg_types.clone();
        let mut parsers: HashMap<&str, fn(&str) -> Box<dyn std::any::Any>> = HashMap::new();
        parsers.insert("i32", |v| Box::new(parse_i32(v)));
        parsers.insert("f32", |v| Box::new(parse_f32(v)));
        parsers.insert("bool", |v| Box::new(parse_bool(v)));
        parsers.insert("String", |v| Box::new(parse_string(v)));

        let func = Box::new(move |args: &[&str]| -> Result<String> {
            if args.len() != 5 {
                return Err(format!("Expected 5 arguments, got {}", args.len()).into());
            }

            let parsed_arg1: $arg1_type = *parsers.get(arg_types[0].as_str()).unwrap()(args[0]).downcast_ref::<$arg1_type>().unwrap();
            let parsed_arg2: $arg2_type = *parsers.get(arg_types[1].as_str()).unwrap()(args[1]).downcast_ref::<$arg2_type>().unwrap();
            let parsed_arg3: $arg3_type = *parsers.get(arg_types[2].as_str()).unwrap()(args[2]).downcast_ref::<$arg3_type>().unwrap();
            let parsed_arg4: $arg4_type = parsers.get(arg_types[3].as_str()).unwrap()(args[3]).downcast_ref::<$arg4_type>().unwrap().clone();
            let parsed_arg5: $arg5_type = *parsers.get(arg_types[4].as_str()).unwrap()(args[4]).downcast_ref::<$arg5_type>().unwrap();

            let result = $func_name(
                parsed_arg1,
                parsed_arg2,
                parsed_arg3,
                &parsed_arg4,
                parsed_arg5,
            );
            Ok(result)
        }) as Box<dyn Fn(&[&str]) -> Result<String> + Send + Sync>;

        Tool {
            name: serde_json::from_str::<Value>($json_description).unwrap()["name"]
                .as_str()
                .unwrap()
                .to_string(),
            function: func,
            arg_names: arg_names,
            arg_types: arg_types_clone,
        }
    }};
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
                    "type": "string",
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
            { "b": "42" },
            { "a": "3.14" },
            { "e": "true" },
            { "d": "example" },
            { "c": "100" }
        ]
    });

    let tool = create_tool_with_function!( fn process_values(b: i32, a: f32, e: bool, d: String, c: i32) -> String, json_description );
    println!("Tool created: {:?}", tool.arg_types);
    println!("Tool created: {:?}", tool.arg_names);
    let result = tool.call(json_input)?;
    println!("{}", result); // Output: Processed: a = 42, b = 3.14, c = true, d = example, e = 100

    Ok(())
}
