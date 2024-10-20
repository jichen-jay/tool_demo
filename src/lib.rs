use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

type MyResult<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

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

pub const PROCESS_VALUE_TOOL_DEF_OBJ: &str = r#"
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

#[derive(Debug, Clone)]
pub enum SupportedType {
    I32(i32),
    F32(f32),
    Bool(bool),
    String(String),
}

// Define Tool struct
#[derive(Clone)]
pub struct Tool {
  pub name: String,
  pub function: Arc<dyn Fn(&[SupportedType]) -> MyResult<String> + Send + Sync>,
  pub tool_def_obj: String,
  pub arg_names: Vec<String>,
  pub arg_types: Vec<String>,
}
impl Tool {
    pub fn call(&self, arguments_w_val: Value) -> MyResult<String> {
        let arguments = arguments_w_val
            .as_object()
            .ok_or("Invalid arguments format")?;
        let mut ordered_vals = Vec::new();

        for (i, arg_name) in self.arg_names.iter().enumerate() {
            let arg_value = if let Some(args) = arguments.get("arguments") {
                // Handle the case where "arguments" is an array
                if let Some(array) = args.as_array() {
                    // Search for the argument in the array
                    let mut found = None;
                    for item in array {
                        if let Some(obj) = item.as_object() {
                            if let Some(value) = obj.get(arg_name) {
                                found = Some(value.clone());
                                break;
                            }
                        }
                    }
                    found.ok_or(format!("Missing argument: {}", arg_name))?
                } else if let Some(obj) = args.as_object() {
                    // Handle the case where "arguments" is an object
                    obj.get(arg_name)
                        .ok_or(format!("Missing argument: {}", arg_name))?
                        .clone()
                } else {
                    return Err("Invalid arguments format".into());
                }
            } else {
                // Try to get the argument from the top level
                arguments
                    .get(arg_name)
                    .ok_or(format!("Missing argument: {}", arg_name))?
                    .clone()
            };

            let parsed_arg = parse_argument(&self.arg_types[i], &arg_value);
            ordered_vals.push(parsed_arg);
        }

        (self.function)(&ordered_vals)
    }
}

pub fn parse_argument(arg_type: &str, arg_value: &Value) -> SupportedType {
    match arg_type {
        "i32" => {
            if let Some(n) = arg_value.as_i64() {
                SupportedType::I32(n as i32)
            } else {
                panic!("Expected i32 for argument");
            }
        }
        "f32" => {
            if let Some(f) = arg_value.as_f64() {
                SupportedType::F32(f as f32)
            } else {
                panic!("Expected f32 for argument");
            }
        }
        "bool" => {
            if let Some(b) = arg_value.as_bool() {
                SupportedType::Bool(b)
            } else if let Some(s) = arg_value.as_str() {
                match s {
                    "true" => SupportedType::Bool(true),
                    "false" => SupportedType::Bool(false),
                    _ => panic!("Expected bool for argument"),
                }
            } else {
                panic!("Expected bool for argument");
            }
        }
        "String" | "&str" => {
            if let Some(s) = arg_value.as_str() {
                SupportedType::String(s.to_string())
            } else {
                panic!("Expected String for argument");
            }
        }
        _ => panic!("Invalid type"),
    }
}

type ParserFn = dyn Fn(SupportedType) -> MyResult<Box<dyn std::any::Any>> + Send + Sync;

pub fn get_parsers() -> HashMap<&'static str, Box<ParserFn>> {
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
                Ok(Box::new(val.clone()) as Box<dyn std::any::Any>)
            } else {
                Err("Type mismatch".into())
            }
        }) as Box<ParserFn>,
    );

    parsers.insert(
        "&str",
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
