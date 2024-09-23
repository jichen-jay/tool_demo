impl Tool {
    fn call(&self, arguments_w_val: Value) -> Result<String> {
        let arguments = arguments_w_val["arguments"].as_array().unwrap();
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
                ordered_vals.push(arg_value.as_str().unwrap());
            } else {
                return Err(format!("Missing argument: {}", arg_name).into());
            }
        }
        println!("Ordered values: {:?}", ordered_vals);

        (self.function)(&ordered_vals)
    }
}

use serde_json::Value;
use std::any::Any;
use std::error::Error;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Debug)]
enum SupportedType {
    I32(i32),
    F32(f32),
    Bool(bool),
    String(String),
}

struct Tool {
    name: String,
    function: Box<dyn Fn(&[&str]) -> Result<String> + Send + Sync>,
    arg_names: Vec<String>,
    arg_types: Vec<String>,
}

fn parse_argument(arg_type: &str, arg_value: &str) -> Result<SupportedType, ParseError> {
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
            _ => Err(ParseError::ParseBoolError),
        },
        "String" => Ok(SupportedType::String(arg_value.to_string())),
        _ => Err(ParseError::InvalidType(arg_type.to_string())),
    }
}

fn downcast_argument<'a>(arg: &'a SupportedType, expected_type: &'a str) -> Result<&'a dyn Any> {
    match (arg, expected_type) {
        (SupportedType::I32(val), "i32") => Ok(val as &dyn Any),
        (SupportedType::F32(val), "f32") => Ok(val as &dyn Any),
        (SupportedType::Bool(val), "bool") => Ok(val as &dyn Any),
        (SupportedType::String(val), "&str") | (SupportedType::String(val), "String") => {
            Ok(val as &dyn Any)
        }
        _ => Err(format!(
            "Type mismatch or unsupported type: expected {}",
            expected_type
        )
        .into()),
    }
}

// fn downcast_argument<'a>(arg: &'a SupportedType, expected_type: &'a str) -> Result<Box<dyn Any + 'a>> {
//     match (arg, expected_type) {
//         (SupportedType::I32(val), "i32") => Ok(Box::new(*val) as Box<dyn Any>),
//         (SupportedType::F32(val), "f32") => Ok(Box::new(*val) as Box<dyn Any>),
//         (SupportedType::Bool(val), "bool") => Ok(Box::new(*val) as Box<dyn Any>),
//         (SupportedType::String(val), "&str") | (SupportedType::String(val), "String") => {
//             Ok(Box::new(val.as_str()) as Box<dyn Any>)
//         }
//         _ => Err(format!("Type mismatch or unsupported type: expected {}", expected_type).into()),
//     }
// }

// fn downcast_argument(arg: &SupportedType) -> Result<&dyn std::any::Any> {
//     match arg {
//         SupportedType::I32(val) => Ok(val as &dyn std::any::Any),
//         SupportedType::F32(val) => Ok(val as &dyn std::any::Any),
//         SupportedType::Bool(val) => Ok(val as &dyn std::any::Any),
//         SupportedType::String(val) => Ok(val as &dyn std::any::Any),
//     }
// }

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

        let func = Box::new(move |args: &[&str]| -> Result<String> {
            if args.len() != 5 {
                return Err(format!("Expected 5 arguments, got {}", args.len()).into());
            }

            let parsed_arg1 = parse_argument(&arg_types[0], args[0])?;
            let parsed_arg2 = parse_argument(&arg_types[1], args[1])?;
            let parsed_arg3 = parse_argument(&arg_types[2], args[2])?;
            let parsed_arg4 = parse_argument(&arg_types[3], args[3])?;
            let parsed_arg5 = parse_argument(&arg_types[4], args[4])?;

            let downcasted_arg1 = match parsed_arg1 {
                SupportedType::I32(val) => val as i32,
                SupportedType::F32(val) => val as f32,
                SupportedType::Bool(val) => val as bool,
                SupportedType::String(val) => val as String,
            };

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
            { "a": "42" },
            { "b": "3.14" },
            { "c": "true" },
            { "d": "example" },
            { "e": "100" }
        ]
    });

    let tool = create_tool_with_function!( fn process_values(a: i32, b: f32, c: bool, d: String, e: i32) -> String, json_description );
    println!("Tool created: {:?}", tool.arg_types);
    println!("Tool created: {:?}", tool.arg_names);
    let result = tool.call(json_input)?;
    println!("{}", result); // Output: Processed: a = 42, b = 3.14, c = true, d = example, e = 100

    Ok(())
}


macro_rules! create_tool_with_function {
    (
        fn $func_name:ident($($arg_name:ident : $arg_type:ty),*) -> $ret_type:ty,
        $json_description:expr
    ) => {{
        let arg_names = vec![
            $(stringify!($arg_name).to_string()),*
        ];
        let arg_types = vec![
            $(stringify!($arg_type).to_string()),*
        ];

        let func = Box::new(move |args: &[&str]| -> Result<String> {
            if args.len() != arg_names.len() {
                return Err(format!("Expected {} arguments, got {}", arg_names.len(), args.len()).into());
            }

            // Parse each argument based on its type
            let parsed_args = (
                $(
                    match parse_argument(&arg_types[args.iter().position(|&x| x == stringify!($arg_name)).unwrap()], args[args.iter().position(|&x| x == stringify!($arg_name)).unwrap()])? {
                        SupportedType::I32(val) => val,
                        SupportedType::F32(val) => val,
                        SupportedType::Bool(val) => val,
                        SupportedType::String(val) => val,
                    }
                ),*
            );

            // Call the function with the parsed arguments
            let result = $func_name(parsed_args);
            Ok(result)
        }) as Box<dyn Fn(&[&str]) -> Result<String> + Send + Sync>;

        Tool {
            name: serde_json::from_str::<Value>($json_description).unwrap()["name"]
                .as_str()
                .unwrap()
                .to_string(),
            function: func,
            arg_names: arg_names,
            arg_types: arg_types,
        }
    }};
}