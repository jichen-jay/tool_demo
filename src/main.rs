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
    description: String,
    parameters: Value,
    function: Box<dyn Fn(&[&str]) -> Result<String> + Send + Sync>,
    //is there a way to save the original function signature? fn get_current_weather(location: &str, unit: &str) -> Result<String>
}

impl Tool {
    fn call(&self, args: &HashMap<String, String>) -> Result<String> {
        let parameters = self.parameters["properties"].as_object().unwrap();
        let mut args = Vec::new();

        for (param_name, _) in parameters {
            match args.get(param_name) {
                Some(value) => args.push(Some(value.as_str())),
                None => {
                    if self.parameters["required"]
                        .as_array()
                        .unwrap()
                        .contains(&Value::String(param_name.clone()))
                    {
                        return Err(anyhow!("Missing required parameter: {}", param_name));
                    } else {
                    }
                }
            }
        }

        (self.function)(&args)
    }
}
#[macro_export]
macro_rules! convert_function {
    ($func:ident, $intermediary:ident, $( $arg:ident ),*) => {
        fn $intermediary(args: &[&str]) -> Result<String> {
            let mut iter = args.iter();
            $(
                let $arg = iter.next().ok_or_else(|| anyhow!("Missing argument: {}", stringify!($arg)))?;
            )*
            $func($($arg),*)
        }
    };
}
#[macro_export]
macro_rules! register_tool {
    ($json:expr, $func:expr) => {{
                                    let tool = create_tool($json, $func);
                                    TOOL_REGISTRY.lock().unwrap().insert(tool.name.clone(), tool);
                                }};
}

fn create_tool<F>(json_description: &str, func: F) -> Tool
where
    F: Fn(&[&str]) -> Result<String> + Send + Sync + 'static,
{
    let description: Value = serde_json::from_str(json_description).unwrap();

    Tool {
        name: description["name"].as_str().unwrap().to_string(),
        description: description["description"].as_str().unwrap().to_string(),
        parameters: description["parameters"].clone(),
        function: Box::new(func),
    }
}

fn get_current_weather(location: &str, unit: &str) -> Result<String> {
    Ok(format!("Weather for {} in {}", location, unit))
}

fn main() -> Result<()> {
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

    //the convention is, convert an original function to a new intermediary function
    //intermediary function name has prepended '_'
    convert_function!(get_current_weather);

    //register the intermediary function with the macro
    register_tool!(&weather_tool_json, _get_current_weather);

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
    let arguments = llm_parsed["arguments"].as_object().unwrap();

    let mut args = HashMap::new();

    //you need to use the original function signature to guarantee the order of the args
    //current logic doesn't use this at all
    //find a way to use this: fn get_current_weather(location: &str, unit: &str) -> Result<String>
    // it has the args listed in order, with type defined
    //for optional arg, if it appears, need to wrap it with Some(), if not, put None instead
    for (key, value) in arguments {
        args.insert(key.clone(), value.as_str().unwrap().to_string());
    }

    let binding = TOOL_REGISTRY.lock().unwrap();
    let weather_tool = binding.get(function_name).unwrap();
    let result = weather_tool.call(&args)?;
    println!("Weather result: {}", result);

    Ok(())
}
