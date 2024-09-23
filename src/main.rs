use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use std::any::Any;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

// Type-specific parse functions
fn parse_i32(arg_value: &str) -> i32 {
    arg_value.parse::<i32>().expect("Expected i32 for argument")
}

fn parse_f32(arg_value: &str) -> f32 {
    arg_value.parse::<f32>().expect("Expected f32 for argument")
}

fn parse_bool(arg_value: &str) -> bool {
    match arg_value {
        "true" => true,
        "false" => false,
        _ => panic!("Expected bool for argument"),
    }
}

fn parse_string(arg_value: &str) -> String {
    arg_value.to_string()
}

// Shared parser map
fn get_parsers() -> HashMap<&'static str, fn(&str) -> Box<dyn Any>> {
    let mut parsers: HashMap<&str, fn(&str) -> Box<dyn Any>> = HashMap::new();
    parsers.insert("i32", |v| Box::new(parse_i32(v)));
    parsers.insert("f32", |v| Box::new(parse_f32(v)));
    parsers.insert("bool", |v| Box::new(parse_bool(v)));
    parsers.insert("String", |v| Box::new(parse_string(v)));
    parsers
}

// Helper function to handle downcasting and cloning
fn downcast_and_clone<T: 'static + Clone>(boxed_value: Box<dyn Any>, arg_type: &str) -> T {
    if arg_type == "String" {
        boxed_value.downcast_ref::<T>().expect("Type mismatch").clone()
    } else {
        *boxed_value.downcast_ref::<T>().expect("Type mismatch")
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
    // Handle function with 1 argument
    (
        fn $func_name:ident($arg1_name:ident : $arg1_type:ty) -> $ret_type:ty,
        $json_description:expr
    ) => {{
        let parsers = get_parsers();
        let arg_name = stringify!($arg1_name).to_string();
        let arg_type = stringify!($arg1_type).to_string();
        let func = Box::new(move |args: &[&str]| -> Result<String> {
            let parse_fn = parsers.get(arg_type.as_str()).expect("Parser not found");
            let boxed_value = parse_fn(args[0]);
            let parsed_arg: $arg1_type = downcast_and_clone(boxed_value, &arg_type);
            let result = $func_name(parsed_arg);
            result
        }) as Box<dyn Fn(&[&str]) -> Result<String> + Send + Sync>;

        Tool {
            name: serde_json::from_str::<Value>($json_description).unwrap()["name"]
                .as_str()
                .unwrap()
                .to_string(),
            function: func,
            arg_names: vec![arg_name],
            arg_types: vec![arg_type],
        }
    }};

    // Handle function with 2 arguments
    (
        fn $func_name:ident($arg1_name:ident : $arg1_type:ty, $arg2_name:ident : $arg2_type:ty) -> $ret_type:ty,
        $json_description:expr
    ) => {{
        let parsers = get_parsers();
        let arg_names = vec![
            stringify!($arg1_name).to_string(),
            stringify!($arg2_name).to_string(),
        ];
        let arg_types = vec![
            stringify!($arg1_type).to_string(),
            stringify!($arg2_type).to_string(),
        ];

        let arg_types_clone = arg_types.clone();

        let func = Box::new(move |args: &[&str]| -> Result<String> {
            let parse_fn1 = parsers.get(arg_types[0].as_str()).expect("Parser not found");
            let boxed_value1 = parse_fn1(args[0]);
            let parsed_arg1: $arg1_type = downcast_and_clone(boxed_value1, &arg_types[0]);

            let parse_fn2 = parsers.get(arg_types[1].as_str()).expect("Parser not found");
            let boxed_value2 = parse_fn2(args[1]);
            let parsed_arg2: $arg2_type = downcast_and_clone(boxed_value2, &arg_types[1]);

            let result = $func_name(parsed_arg1, parsed_arg2);
            result
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

    // Handle function with 3 arguments
    (
        fn $func_name:ident($arg1_name:ident : $arg1_type:ty, $arg2_name:ident : $arg2_type:ty, $arg3_name:ident : $arg3_type:ty) -> $ret_type:ty,
        $json_description:expr
    ) => {{
        let parsers = get_parsers();
        let arg_names = vec![
            stringify!($arg1_name).to_string(),
            stringify!($arg2_name).to_string(),
            stringify!($arg3_name).to_string(),
        ];
        let arg_types = vec![
            stringify!($arg1_type).to_string(),
            stringify!($arg2_type).to_string(),
            stringify!($arg3_type).to_string(),
        ];
        let arg_types_clone = arg_types.clone();

        let func = Box::new(move |args: &[&str]| -> Result<String> {
            let parse_fn1 = parsers.get(arg_types[0].as_str()).expect("Parser not found");
            let boxed_value1 = parse_fn1(args[0]);
            let parsed_arg1: $arg1_type = downcast_and_clone(boxed_value1, &arg_types[0]);

            let parse_fn2 = parsers.get(arg_types[1].as_str()).expect("Parser not found");
            let boxed_value2 = parse_fn2(args[1]);
            let parsed_arg2: $arg2_type = downcast_and_clone(boxed_value2, &arg_types[1]);

            let parse_fn3 = parsers.get(arg_types[2].as_str()).expect("Parser not found");
            let boxed_value3 = parse_fn3(args[2]);
            let parsed_arg3: $arg3_type = downcast_and_clone(boxed_value3, &arg_types[2]);

            let result = $func_name(parsed_arg1, parsed_arg2, parsed_arg3);
            result
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

    // Handle function with 4 arguments
    (
        fn $func_name:ident($arg1_name:ident : $arg1_type:ty, $arg2_name:ident : $arg2_type:ty, $arg3_name:ident : $arg3_type:ty, $arg4_name:ident : $arg4_type:ty) -> $ret_type:ty,
        $json_description:expr
    ) => {{
        let parsers = get_parsers();
        let arg_names = vec![
            stringify!($arg1_name).to_string(),
            stringify!($arg2_name).to_string(),
            stringify!($arg3_name).to_string(),
            stringify!($arg4_name).to_string(),
        ];
        let arg_types = vec![
            stringify!($arg1_type).to_string(),
            stringify!($arg2_type).to_string(),
            stringify!($arg3_type).to_string(),
            stringify!($arg4_type).to_string(),
        ];
        let arg_types_clone = arg_types.clone();

        let func = Box::new(move |args: &[&str]| -> Result<String> {
            let parse_fn1 = parsers.get(arg_types[0].as_str()).expect("Parser not found");
            let boxed_value1 = parse_fn1(args[0]);
            let parsed_arg1: $arg1_type = downcast_and_clone(boxed_value1, &arg_types[0]);

            let parse_fn2 = parsers.get(arg_types[1].as_str()).expect("Parser not found");
            let boxed_value2 = parse_fn2(args[1]);
            let parsed_arg2: $arg2_type = downcast_and_clone(boxed_value2, &arg_types[1]);

            let parse_fn3 = parsers.get(arg_types[2].as_str()).expect("Parser not found");
            let boxed_value3 = parse_fn3(args[2]);
            let parsed_arg3: $arg3_type = downcast_and_clone(boxed_value3, &arg_types[2]);

            let parse_fn4 = parsers.get(arg_types[3].as_str()).expect("Parser not found");
            let boxed_value4 = parse_fn4(args[3]);
            let parsed_arg4: $arg4_type = downcast_and_clone(boxed_value4, &arg_types[3]);

            let result = $func_name(parsed_arg1, parsed_arg2, parsed_arg3, parsed_arg4);
            result
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

    // Handle function with 5 arguments
    (
        fn $func_name:ident($arg1_name:ident : $arg1_type:ty, $arg2_name:ident : $arg2_type:ty, $arg3_name:ident : $arg3_type:ty, $arg4_name:ident : $arg4_type:ty, $arg5_name:ident : $arg5_type:ty) -> $ret_type:ty,
        $json_description:expr
    ) => {{
        let parsers = get_parsers();
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
            let parse_fn1 = parsers.get(arg_types[0].as_str()).expect("Parser not found");
            let boxed_value1 = parse_fn1(args[0]);
            let parsed_arg1: $arg1_type = downcast_and_clone(boxed_value1, &arg_types[0]);

            let parse_fn2 = parsers.get(arg_types[1].as_str()).expect("Parser not found");
            let boxed_value2 = parse_fn2(args[1]);
            let parsed_arg2: $arg2_type = downcast_and_clone(boxed_value2, &arg_types[1]);

            let parse_fn3 = parsers.get(arg_types[2].as_str()).expect("Parser not found");
            let boxed_value3 = parse_fn3(args[2]);
            let parsed_arg3: $arg3_type = downcast_and_clone(boxed_value3, &arg_types[2]);

            let parse_fn4 = parsers.get(arg_types[3].as_str()).expect("Parser not found");
            let boxed_value4 = parse_fn4(args[3]);
            let parsed_arg4: $arg4_type = downcast_and_clone(boxed_value4, &arg_types[3]);

            let parse_fn5 = parsers.get(arg_types[4].as_str()).expect("Parser not found");
            let boxed_value5 = parse_fn5(args[4]);
            let parsed_arg5: $arg5_type = downcast_and_clone(boxed_value5, &arg_types[4]);

            let result = $func_name(parsed_arg1, parsed_arg2, parsed_arg3, parsed_arg4, parsed_arg5);
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





fn get_current_weather(location: String, unit: String) -> Result<String> {
    Ok(format!("Weather for {} in {}", location, unit))
}

fn process_values(a: i32, b: f32, c: bool, d: &str, e: i32) -> String {
    format!(
        "Processed: a = {}, b = {}, c = {}, d = {}, e = {}",
        a, b, c, d, e
    )
}
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

//     let json_input = serde_json::json!({
//         "arguments": [
//             { "location": "SFO" },
//             { "unit": "celsius" },
//         ]
//     });

//     let tool = create_tool_with_function!(fn get_current_weather(location: String, unit: String) -> Result<String>,
//     weather_tool_json     );
//     println!("Tool created: {:?}", tool.arg_types);
//     println!("Tool created: {:?}", tool.arg_names);
//     let result = tool.call(json_input)?;
//     println!("{}", result); // O

//     Ok(())
// }

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
