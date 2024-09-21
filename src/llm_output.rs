fn llm_output() -> anyhow::Result<()> {
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
    // convert_function!(get_current_weather);

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

    let mut args = Vec::new();
    for arg in ARG_NAMES {
        args.push(arguments.get(*arg).unwrap().to_string());
    }
    let args_ref: Vec<&str> = args.iter().map(|v| v.as_str()).collect();

    let result = _get_current_weather(&args_ref)?;
    println!("Weather result: {}", result);
    Ok(())
}

fn get_current_weather(location: &str, unit: &str) -> Result<String> {
    Ok(format!("Weather for {} in {}", location, unit))
}
macro_rules! extract_args_and_types {
    // Match the function signature and extract argument names and types
    (
        fn $func_name:ident($($arg_name:ident : $arg_type:ty),*) -> $ret_type:ty
    ) => {
        // Create arrays for argument names and types
        const ARG_NAMES: &[&str] = &[$(stringify!($arg_name)),*];
        const ARG_TYPES: &[&str] = &[$(stringify!($arg_type)),*];
    };
}
