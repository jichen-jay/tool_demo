use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use anyhow::Result;

use func_builder::create_tool_with_function; // Replace `your_crate_name` with the actual name

type MyResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Debug, Clone)]
enum SupportedType {
    I32(i32),
    F32(f32),
    Bool(bool),
    String(String),
}

// Define Tool struct
#[derive(Clone)]
struct Tool {
    name: String,
    function: Arc<dyn Fn(&[SupportedType]) -> MyResult<String> + Send + Sync>,
    tool_def_obj: &'static str,
    arg_names: Vec<String>,
    arg_types: Vec<String>,
}

impl Tool {
    fn call(&self, arguments_w_val: Value) -> MyResult<String> {
        let arguments = arguments_w_val
            .as_object()
            .ok_or("Invalid arguments format")?;
        let mut ordered_vals = Vec::new();

        for (i, arg_name) in self.arg_names.iter().enumerate() {
            if let Some(arg_value) = arguments.get(arg_name) {
                let arg_str = arg_value.as_str().ok_or("Invalid argument value")?;
                let parsed_arg = parse_argument(&self.arg_types[i], arg_str);
                ordered_vals.push(parsed_arg);
            } else {
                return Err(format!("Missing argument: {}", arg_name).into());
            }
        }

        (self.function)(&ordered_vals)
    }
}

// Parser functions
fn parse_argument(arg_type: &str, arg_value: &str) -> SupportedType {
    match arg_type {
        "i32" => SupportedType::I32(arg_value.parse::<i32>().expect("Expected i32 for argument")),
        "f32" => SupportedType::F32(arg_value.parse::<f32>().expect("Expected f32 for argument")),
        "bool" => SupportedType::Bool(match arg_value {
            "true" => true,
            "false" => false,
            _ => panic!("Expected bool for argument"),
        }),
        "String" => SupportedType::String(arg_value.to_string()),
        "& str" => SupportedType::String(arg_value.to_string()),
        _ => panic!("Invalid type"),
    }
}

// Get parsers
type ParserFn = dyn Fn(SupportedType) -> MyResult<Box<dyn std::any::Any>> + Send + Sync;

fn get_parsers() -> HashMap<&'static str, Box<ParserFn>> {
    let mut parsers = HashMap::new();

    parsers.insert(
        "i32",
        Box::new(|v| {
            if let SupportedType::I32(val) = v {
                Ok(Box::new(val) as Box<dyn std::any::Any>)
            } else {
                Err("Type mismatch".into())
            }
        }) as Box<ParserFn>,
    );

    parsers.insert(
        "f32",
        Box::new(|v| {
            if let SupportedType::F32(val) = v {
                Ok(Box::new(val) as Box<dyn std::any::Any>)
            } else {
                Err("Type mismatch".into())
            }
        }) as Box<ParserFn>,
    );

    parsers.insert(
        "bool",
        Box::new(|v| {
            if let SupportedType::Bool(val) = v {
                Ok(Box::new(val) as Box<dyn std::any::Any>)
            } else {
                Err("Type mismatch".into())
            }
        }) as Box<ParserFn>,
    );

    parsers.insert(
        "String",
        Box::new(|v| {
            if let SupportedType::String(val) = v {
                Ok(Box::new(val) as Box<dyn std::any::Any>)
            } else {
                Err("Type mismatch".into())
            }
        }) as Box<ParserFn>,
    );

    parsers.insert(
        "& str",
        Box::new(|v| {
            if let SupportedType::String(val) = v {
                Ok(Box::new(val.to_string()) as Box<dyn std::any::Any>)
            } else {
                Err("Type mismatch".into())
            }
        }) as Box<ParserFn>,
    );

    parsers
}

#[create_tool_with_function(GET_WEATHER_TOOL_DEF_OBJ)]
fn get_current_weather(location: String, unit: String) -> MyResult<String> {
    if location.contains("New") {
        Ok(format!("Weather for {} in {}", location, unit))
    } else {
        Err(format!("Weather for {} in {}", location, unit).into())
    }
}

pub const GET_WEATHER_TOOL_DEF_OBJ: &str = r#"
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

fn main() -> Result<(), Box<dyn Error>> {
    // Create the tool
    let tool = create_tool();

    // Simulate LLM output
    let llm_output = serde_json::json!({
        "location": "York, NY",
        "unit": "fahrenheit"
    });

    // Call the tool with the arguments
    let result = tool.call(llm_output).expect("funccall");

    println!("Result: {}", result);

    Ok(())
}
