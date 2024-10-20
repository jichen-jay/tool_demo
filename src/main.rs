use func_builder::create_tool_with_function;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Mutex;
use tool_demo::{
    get_parsers, SupportedType, Tool, GET_WEATHER_TOOL_DEF_OBJ, PROCESS_VALUE_TOOL_DEF_OBJ,
};

type MyResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub static STORE: Lazy<Mutex<HashMap<String, Tool>>> = Lazy::new(|| Mutex::new(HashMap::new()));

#[create_tool_with_function(PROCESS_VALUE_TOOL_DEF_OBJ)]
fn process_values(a: i32, b: f32, c: bool, d: String, e: i32) -> MyResult<String> {
    if a > 10 {
        Ok(format!(
            "Processed: a = {}, b = {}, c = {}, d = {}, e = {}",
            a, b, c, d, e
        ))
    } else {
        Err(format!(
            "Processed: a = {}, b = {}, c = {}, d = {}, e = {}",
            a, b, c, d, e
        )
        .into())
    }
}

#[create_tool_with_function(GET_WEATHER_TOOL_DEF_OBJ)]
fn get_current_weather(location: String, unit: String) -> MyResult<String> {
    if location.contains("New") {
        Ok(format!("Weather for {} in {}", location, unit))
    } else {
        Err(format!("Weather for {} in {}", location, unit).into())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Create the tool
    let store = STORE.lock().unwrap();

    let llm_output = serde_json::json!({
        "location": "York, NY",
        "unit": "fahrenheit"
    }); // Use your tools as needed

    println!("tool sig : {:?}", llm_output.clone());

    if let Some(tool) = store.get("get_current_weather") {
        println!("tool sig : {:?}", tool.name.clone());

        match tool.call(llm_output) {
            Ok(result) => println!("Result: {}", result),
            Err(e) => eprintln!("Error: {e}"),
        }
    }

    // match tool.call(llm_output) {
    //     Ok(result) => println!("Result: {}", result),
    //     Err(e) => eprintln!("Error: {e}"),
    // }
    // let json_input = serde_json::json!({
    //     "arguments": [
    //         { "a": 20 },
    //         { "b": 3.14 },
    //         { "c": "true" },
    //         { "d": "example" },
    //         { "e": 100 }
    //     ]
    // });
    // let tool = &PROCESS_VALUES_TOOL;

    // match tool.call(json_input) {
    //     Ok(result) => println!("Result: {}", result),
    //     Err(e) => eprintln!("Error: {e}"),
    // }

    Ok(())
}
