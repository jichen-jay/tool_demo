use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Mutex;

lazy_static! {
    static ref TOOL_REGISTRY: Mutex<HashMap<String, Tool>> = Mutex::new(HashMap::new());
}

struct Tool {
    name: String,
    function: Box<dyn Fn(&[&str]) -> Result<String> + Send + Sync>,
    args: Vec<String>,      // Argument names
    args_type: Vec<String>, // Argument types
}

impl Tool {
    fn call(&self, args: Vec<String>) -> Result<String> {
        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        (self.function)(&args_ref)
    }
}
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

#[macro_export]
macro_rules! register_tool {
    ($json:expr, fn $func:ident($($arg_name:ident : $arg_type:ty),*) -> $ret_type:ty) => {{
        // Extract argument names and types
        let arg_names = vec![$(stringify!($arg_name).to_string()),*];
        let arg_types = vec![$(stringify!($arg_type).to_string()),*];

        // Convert the function
        let intermediary = convert_function!($func, arg_names);

        // Create and register the tool
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
        args: arg_names,
        args_type: arg_types,
        function: Box::new(func),
    }
}

fn get_current_weather(location: &str, unit: &str) -> Result<String> {
    Ok(format!("Weather for {} in {}", location, unit))
}

fn main() -> anyhow::Result<()> {
    let weather_tool_json = r#"
    {
        "name": "get_current_weather",
        "description": "Get the current weather in a given location",
        "parameters": {
            "type": "object",
            "properties": {
                "location": {
                    "type": "string",
                    "description": "The city and state, e.g. San Francisco, CA"
                },
                "unit": {
                    "type": "string",
                    "enum": ["celsius", "fahrenheit"],
                    "description": "The unit of measurement"
                }
            },
            "required": ["location", "unit"]
        }
    }
    "#;

    register_tool!(weather_tool_json, fn get_current_weather(location: &str, unit: &str) -> Result<String>);

    let llm_output = r#"
    {
        "arguments": {
            "location": "Glasgow, Scotland",
            "unit": "celsius"
        },
        "name": "get_current_weather"
    }
    "#;

    let llm_parsed: Value = serde_json::from_str(llm_output)?;
    let function_name = llm_parsed["name"].as_str().unwrap();
    let arguments = llm_parsed["arguments"]
        .as_object()
        .ok_or_else(|| anyhow!("Expected 'arguments' to be an object"))?;

    let registry = TOOL_REGISTRY.lock().unwrap();
    let function = registry.get(function_name).unwrap();

    let mut args = Vec::new();
    for arg in &function.args {
        args.push(arguments.get(arg.as_str()).unwrap().to_string());
    }

    let result = function.call(args)?;
    println!("Weather result: {}", result);
    Ok(())
}
