use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

type MyResult<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[macro_export]
macro_rules! create_tool_with_function {
    (
        fn $func_name:ident($($arg_name:ident : $arg_type:ty),*) -> $ret_type:ty,
        $TOOL_DEF_OBJ:expr
    ) => {{
        let arg_names = vec![$(stringify!($arg_name).to_string()),*];
        let arg_types = vec![$(stringify!($arg_type).to_string()),*];

        let func = Arc::new(move |args: &[SupportedType]| -> MyResult<String> {
            let parsers = get_parsers();

            let mut iter = args.iter();
            $(
                let $arg_name = {
                    let arg = iter.next().ok_or("Not enough arguments")?.clone();
                    let parser = parsers.get(stringify!($arg_type)).ok_or("Parser not found")?;
                    let any_val = parser(arg)?;
                    let val = any_val.downcast::<$arg_type>().map_err(|_| "Type mismatch")?;
                    *val
                };
            )*

            $func_name($($arg_name),*)
        }) as Arc<dyn Fn(&[SupportedType]) -> MyResult<String> + Send + Sync>;

        Tool {
            name: serde_json::from_str::<Value>($TOOL_DEF_OBJ)
                .unwrap()["name"]
                .as_str()
                .unwrap()
                .to_string(),
            function: func,
            arg_names: arg_names,
            arg_types: arg_types,
        }
    }};
}

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

// Implement SupportedType and parsers
#[derive(Debug, Clone)]
enum SupportedType {
    I32(i32),
    F32(f32),
    Bool(bool),
    String(String),
}

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

fn parse_argument(arg_type: &str, arg_value: &str) -> SupportedType {
    match arg_type {
        "i32" => SupportedType::I32(parse_i32(arg_value)),
        "f32" => SupportedType::F32(parse_f32(arg_value)),
        "bool" => SupportedType::Bool(parse_bool(arg_value)),
        "String" | "&str" => SupportedType::String(parse_string(arg_value)),
        _ => panic!("Invalid type"),
    }
}

// Define the Tool struct with Arc and derive Clone
#[derive(Clone)]
struct Tool {
    name: String,
    function: Arc<dyn Fn(&[SupportedType]) -> MyResult<String> + Send + Sync>,
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

// Demonstrate how to use the generated tool
fn main() -> Result<(), Box<dyn Error>> {
    let tool = create_tool_with_function!(
        fn get_current_weather(location: String, unit: String) -> MyResult<String>,
        GET_WEATHER_TOOL_DEF_OBJ
    );
    let llm_output = serde_json::json!({
        "location": "York, NY",
        "unit": "fahrenheit"
    });

    match tool.call(llm_output) {
        Ok(result) => println!("Result: {}", result),
        Err(e) => eprintln!("Error: {e}"),
    }

    Ok(())
}
