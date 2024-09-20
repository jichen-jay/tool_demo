use anyhow::{anyhow, Result};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{Mutex, Arc};

#[derive(Deserialize, Serialize, Clone)]
pub struct ParameterDefinition {
    #[serde(rename = "type")]
    param_type: String,
    description: Option<String>,
    #[serde(rename = "enum")]
    enum_values: Option<Vec<String>>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct ToolDefinition {
    name: String,
    description: String,
    parameters: Value,
}
pub struct ToolRegistry {
    tools: HashMap<String, Tool>,
}

impl ToolRegistry {
    fn new() -> Self {
        Self { tools: HashMap::new() }
    }

    fn register(&mut self, tool: Tool) {
        self.tools.insert(tool.definition.name.clone(), tool);
    }

    fn unregister(&mut self, name: &str) {
        self.tools.remove(name);
    }

    fn get(&self, name: &str) -> Option<&Tool> {
        self.tools.get(name)
    }

  
}

pub struct Tool {
    pub definition: ToolDefinition,
    function: Arc<dyn Fn(&HashMap<String, String>) -> Result<String> + Send + Sync>,
}
impl Clone for Tool {
    fn clone(&self) -> Self {
        Tool {
            definition: self.definition.clone(),
            function: Arc::clone(&self.function),
        }
    }
}

impl Tool {
    pub fn new<F>(definition: ToolDefinition, function: F) -> Self
    where
        F: Fn(&HashMap<String, String>) -> Result<String> + Send + Sync + 'static,
    {
        Tool {
            definition,
            function: Arc::new(function),
        }
    }

    pub fn call(&self, args: &HashMap<String, String>) -> Result<String> {
        (self.function)(args)
    }
}
static GLOBAL_REGISTRY: Lazy<Mutex<ToolRegistry>> = Lazy::new(|| Mutex::new(ToolRegistry::new()));

pub fn register_tool_from_json<F>(json_str: &str, function: F) -> Result<()>
where
    F: Fn(&HashMap<String, String>) -> Result<String> + Send + Sync + 'static,
{
    let definition: ToolDefinition = serde_json::from_str(json_str)?;
    let tool = Tool::new(definition, function);
    let mut registry = GLOBAL_REGISTRY.lock().unwrap();
    registry.register(tool);
    Ok(())
}

macro_rules! generate_typed_function {
    ($name:ident) => {
        pub fn $name(args: &HashMap<String, String>) -> Result<String> {
            let tool = get_tool(stringify!($name))
                .ok_or_else(|| anyhow!("Tool not found: {}", stringify!($name)))?;

            let params: HashMap<String, ParameterDefinition> =
                serde_json::from_value(tool.definition.parameters["properties"].clone())?;
            let required: Vec<String> =
                serde_json::from_value(tool.definition.parameters["required"].clone())?;

            for req in required {
                if !args.contains_key(&req) {
                    return Err(anyhow!("Missing required parameter: {}", req));
                }
            }

            for (name, value) in args {
                if let Some(param_def) = params.get(name) {
                    match param_def.param_type.as_str() {
                        "string" => {
                            if let Some(enum_values) = &param_def.enum_values {
                                if !enum_values.contains(value) {
                                    return Err(anyhow!("Invalid value for {}: {}", name, value));
                                }
                            }
                        }
                        _ => {
                            return Err(anyhow!(
                                "Unsupported parameter type: {}",
                                param_def.param_type
                            ))
                        }
                    }
                } else {
                    return Err(anyhow!("Unknown parameter: {}", name));
                }
            }

            tool.call(args)
        }
    };
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
                    "enum": ["celsius", "fahrenheit"]
                }
            },
            "required": ["location"]
        }
    }
    "#;
    register_tool_from_json(weather_tool_json, |args| {
        let location = args.get("location").ok_or_else(|| anyhow!("Missing location"))?;
        let unit = args.get("unit").unwrap_or(&"celsius".to_string());
        get_current_weather(args)
    })?;
    
    generate_typed_function!(get_current_weather);
    
    // Usage remains the same
    let mut args = HashMap::new();
    args.insert("location".to_string(), "San Francisco, CA".to_string());
    args.insert("unit".to_string(), "fahrenheit".to_string());
    let result = get_current_weather(&args)?;
    println!("Result: {}", result);



    Ok(())
}

pub fn get_current_weather(location: &str, unit: &str) -> Result<String> {
    Ok("fake_weather".to_string())
}
pub fn get_tool(name: &str) -> Option<Tool> {
    let registry = GLOBAL_REGISTRY.lock().unwrap();
    registry.get(name).cloned()
}