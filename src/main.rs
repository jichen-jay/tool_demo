use anyhow::{anyhow, Result};
use serde_json::Value;
use std::collections::HashMap;

type DynFunction = Box<dyn Fn(&HashMap<String, String>) -> Result<String>>;

struct Tool {
    name: String,
    description: String,
    parameters: Value,
    function: DynFunction,
}

impl Tool {
    fn call(&self, args: &HashMap<String, String>) -> Result<String> {
        (self.function)(args)
    }
}

fn register_tool<F>(json_description: &str, func: F) -> Tool 
where
    F: Fn(&[&str]) -> Result<String> + 'static,
{
    let description: Value = serde_json::from_str(json_description).unwrap();
    let parameters = description["parameters"]["properties"].as_object().unwrap();
    let required: Vec<String> = description["parameters"]["required"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();

    let function: DynFunction = Box::new(move |args: &HashMap<String, String>| {
        let mut func_args = Vec::new();
        for param in required.iter() {
            let arg = args.get(param).ok_or_else(|| anyhow!("Missing parameter: {}", param))?;
            func_args.push(arg.as_str());
        }
        func(&func_args)
    });

    Tool {
        name: description["name"].as_str().unwrap().to_string(),
        description: description["description"].as_str().unwrap().to_string(),
        parameters: description["parameters"].clone(),
        function,
    }
}

// Example functions with different numbers of arguments
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


    let weather_tool = register_tool(weather_tool_json, |args| {
        get_current_weather(args[0], args[1])
    });

    // Use the weather tool
    let mut args = HashMap::new();
    args.insert("location".to_string(), "San Francisco, CA".to_string());
    args.insert("unit".to_string(), "fahrenheit".to_string());
    let result = weather_tool.call(&args)?;
    println!("Weather result: {}", result);


    Ok(())
}
