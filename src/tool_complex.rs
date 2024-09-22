#[macro_export]
macro_rules! register_tool {
    ($json:expr, fn $func:ident($($arg_name:ident : $arg_type:ty),*) -> $ret_type:ty) => {{
        let arg_names = vec![$(stringify!($arg_name).to_string()),*];
        let arg_types = vec![$(stringify!($arg_type).to_string()),*];

        let intermediary = convert_function!($func, arg_names, arg_types);

        let tool = create_tool($json, intermediary, arg_names, arg_types);
        TOOL_REGISTRY.lock().unwrap().insert(tool.name.clone(), tool);
    }};
}

fn create_tool<F>(
    json_description: &str,
    func: F,
    arg_names: Vec<String>,
    arg_types: Vec<String>,
) -> Tool
where
    F: Fn(&[&str]) -> Result<String> + Send + Sync + 'static,
{
    let description: Value = serde_json::from_str(json_description).unwrap();
    Tool {
        name: description["name"].as_str().unwrap().to_string(),
        arg_names: arg_names,
        args_type: arg_types,
        function: Box::new(func),
    }
}
use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use serde_json::Value;
use std::any::Any;
use std::collections::HashMap;
use std::sync::Mutex;

lazy_static! {
    static ref TOOL_REGISTRY: Mutex<HashMap<String, Tool>> = Mutex::new(HashMap::new());
}

struct Tool {
    name: String,
    function: Box<dyn Fn(&[&str]) -> Result<String> + Send + Sync>,
    arg_names: Vec<String>,
    args_type: Vec<String>,
}

impl Tool {
    fn call(&self, arguments_w_val: Value) -> Result<String> {
        let arguments = arguments_w_val["arguments"].as_array().unwrap();

        let arg_map = arguments.into_iter()
            .map(|arg| arg.as_object().unwrap().iter().next().unwrap())
            .collect::<HashMap<String, String>>();

        let mut ordered_vals = Vec::new();

        for (i, (arg_name, arg_type)) in
            self.arg_names.iter().zip(self.arg_types.iter()).enumerate()
        {
            let arg_value = arg_map.get(&arg_name).unwrap();
            let converted_val = convert_value(arg_type, arg_value)?;
            ordered_vals.push(converted_val);
        }
        let args: Vec<&str> = vec![
            arg_map.get("a").unwrap(),
            arg_map.get("b").unwrap(),
            arg_map.get("c").unwrap(),
            arg_map.get("d").unwrap(),
            arg_map.get("e").unwrap(),
        ];
   // this is the real func I want to call: fn process_values(a: i32, b: f32, c: bool, d: &str, e: i32) -> String 

    }
}
fn convert_value(arg_type: &str, arg_value: &str) -> Result<Box<dyn Any>> {
    match arg_type {
        "i32" => Ok(Box::new(arg_value.parse::<i32>().map_err(|e| {
            anyhow!("Failed to parse '{}' as i32: {}", arg_value, e)
        })?)),
        "f32" => Ok(Box::new(arg_value.parse::<f32>().map_err(|e| {
            anyhow!("Failed to parse '{}' as f32: {}", arg_value, e)
        })?)),
        "bool" => Ok(Box::new(arg_value.parse::<bool>().map_err(|e| {
            anyhow!("Failed to parse '{}' as bool: {}", arg_value, e)
        })?)),
        "&str" | "String" => Ok(Box::new(arg_value.to_string())),
        _ => Err(anyhow!("Unsupported argument type: {}", arg_type)),
    }
}

#[macro_export]
macro_rules! convert_function {
    ($func:ident, $arg_names:expr, $arg_types:expr) => {{
        let arg_names = $arg_names.clone();
        let arg_types = $arg_types.clone();

        Box::new(move |arg_names: &[&str]| -> Result<String> {
            if arg_names.len() != arg_names.len() {
                return Err(anyhow!("Incorrect number of arguments_w_val"));
            }

            let mut converted_args: Vec<Box<dyn Any>> = Vec::new();

            for (i, (arg_name, arg_type)) in arg_names.iter().zip(arg_types.iter()).enumerate() {
                let arg_value = arg_map.get(&arg_name).unwrap();
                let converted_arg = convert_value(arg_type, arg_value)?;
                converted_args.push(converted_arg);
            }

            let result = match converted_args.as_slice() {
                [a, b, c, d, e] => {
                    let a = a.downcast_ref::<i32>().unwrap();
                    let b = b.downcast_ref::<f32>().unwrap();
                    let c = c.downcast_ref::<bool>().unwrap();
                    let d = d.downcast_ref::<String>().unwrap();
                    let e = e.downcast_ref::<i32>().unwrap();
                    $func(*a, *b, *c, d, *e)
                }
                _ => return Err(anyhow!("Unsupported number of arguments_w_val")),
            };

            Ok(result)
        }) as Box<dyn Fn(&[&str]) -> Result<String> + Send + Sync>
    }};
}

fn process_values(a: i32, b: f32, c: bool, d: &str, e: i32) -> String {
    format!(
        "Processed: a = {}, b = {}, c = {}, d = {}, e = {}",
        a, b, c, d, e
    )
}

fn main() -> anyhow::Result<()> {
    let tool_json = r#"
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

    register_tool!(tool_json, fn process_values(a: i32, b: f32, c: bool, d: &str, e: i32) -> String);

    let llm_output = r#"
    {
        "arguments_w_val": [
            {"e": "100"},
            {"a": "42"},
            {"c": "true"},
            {"b": "3.14"},
            {"d": "Hello world"}
        ],
        "name": "process_values"
    }
    "#;
    let llm_parsed: Value = serde_json::from_str(llm_output)?;
    let function_name = llm_parsed["name"].as_str().unwrap();
    let arguments_w_val = llm_parsed["arguments_w_val"].as_array().unwrap();

    let mut arg_map = HashMap::new();
    for arg in arguments_w_val {
        let (key, value) = arg.as_object().unwrap().iter().next().unwrap();
        arg_map.insert(key.clone(), value.as_str().unwrap());
    }

    // Extract arguments_w_val by name
    let args: Vec<&str> = vec![
        arg_map.get("a").unwrap(),
        arg_map.get("b").unwrap(),
        arg_map.get("c").unwrap(),
        arg_map.get("d").unwrap(),
        arg_map.get("e").unwrap(),
    ];

    let registry = TOOL_REGISTRY.lock().unwrap();
    let function = registry.get(function_name).unwrap();

    let result = function.call(&args)?;
    println!("Result: {}", result);

    Ok(())
}

fn get_current_weather(location: &str, unit: &str) -> Result<String, String> {}
