#[macro_export]
macro_rules! register_tool {

//the json input is a Value object that encapsulate all type information, description of the function and the scenario under which the upstream ops shall invoke it
    // this is an example of function to be registered: fn process_values(a: i32, b: f32, c: bool, d: &str, e: i32) -> String
    ($json:expr, fn $func:ident($($arg_name:ident : $arg_type:ty),*) -> $ret_type:ty) => {{
        let arg_names = vec![$(stringify!($arg_name).to_string()),*];
        let arg_types = vec![$(stringify!($arg_type).to_string()),*];
    
    
    // the intermediary is a function that takes string as input and output: fn process_values_itermediary([a, b, c, d, e]: &[&str] -> string
    // this way, a function can be invoked dynamically
    // the intermediary function, represented by a long string, encapsulating everything: function name, args, output, 
    // without showing type information, is the best the text processing segment of the code can do to represent an arbitrary function
    // with that textual representation of the function, and the defined logic, we're able to run the original function
    //otherwise, the original function can't be invoked easily by some string output of the upstream ops
        let intermediary = convert_function!($func, arg_names, arg_types);

        // a tool establishes the association of the intermediary function that can be invoked according a text output of upstream ops and the real Rust function that runs with args precisely typed
        // the challenge is to maintain the type info of the args in the original function in the intermediary function
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
    F: Fn(&[&str]) -> Result<&str> + Send + Sync + 'static,
{
    let description: Value = serde_json::from_str(json_description).unwrap();
    Tool {
        name: description["name"].as_str().unwrap().to_string(),
        args: arg_names,
        args_type: arg_types,
        //the func is the intermdiary function readily invoked by textual output of the uptream ops
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
    function: Box<dyn Fn(&[&str]) -> Result<&str> + Send + Sync>,
    args: Vec<String>,
    args_type: Vec<String>,
}

impl Tool {
    //not sure whether args is best represented as below, or a String to be parsed into 1 or multiple args of certain types
    fn call(&self, args: [&str]) -> Result<&str> {
        (self.function)(&args)
    }
}

//it serves to convert the args in string format to typed data needed by the original Rust function
//needs to put the args in Value format somewhere
fn convert_value( arg_type: &str, arg_name: &str) -> Result<Box<dyn Any>> {
    match arg_type {
        "i32" => Ok(Box::new(
            arg_name.as_i64()
                .ok_or_else(|| anyhow!("Expected i32 for {}", arg_name))? as i32,
        )),
        "f32" => Ok(Box::new(
            arg_name.as_f64()
                .ok_or_else(|| anyhow!("Expected f32 for {}", arg_name))? as f32,
        )),
        "bool" => {
            Ok(Box::new(arg_name.as_bool().ok_or_else(|| {
                anyhow!("Expected bool for {}", arg_name)
            })?))
        }
        "&str" | "String" => Ok(Box::new(
            arg_name.as_str()
                .ok_or_else(|| anyhow!("Expected string for {}", arg_name))?
                .to_string(),
        )),
        _ => Err(anyhow!("Unsupported argument type: {}", arg_type)),
    }
}

//aims to convert an orginal function whose arg types are precisedly defined to an intermediary function whose input and output are string
//converted function can be easily invoked with the textual output of the uptream ops, there is no easy way to invoke the original function like this
#[macro_export]
macro_rules! convert_function {
    ($func:ident, $arg_names:expr, $arg_types:expr) => {{
        let arg_names = $arg_names.clone();
        let arg_types = $arg_types.clone();

        //here I'm trying to convert args according to their types, am not exactly sure how this would work
        let mut ordered_args: Vec<&Box<dyn Any>> = Vec::new();

        for (arg_name, arg_type) in arg_names.iter().zip(arg_types.iter()) {
            let arg = args
                .iter()
                .find(|a| a.is_object() && a.as_object().unwrap().contains_key(arg_name))
                .and_then(|a| a.get(arg_name))
                .ok_or_else(|| anyhow!("Missing argument: {}", arg_name))?;

            let converted_arg = convert_value(arg, arg_type, arg_name)?;
            ordered_args.push(converted_arg);
        }

        //I imagine assembling the intermediary function according to the number of args, 
        //a function with 1 arg will use a 1 arg macro to build, and so on.
        match ordered_args.len() {
            1 => {
                let [a] = ordered_args;

                $func(a);
            }
            2 => {
                let [a,b] = ordered_args;

                $func(a,b);
            }
            3 => call_func_3!($func, ordered_args),
            4 => call_func_4!($func, ordered_args),
            5 => call_func_5!($func, ordered_args),
        }

) as Box<dyn Fn(&[&str]) -> Result<&str> + Send + Sync>
    }};
}

// Example function that takes 5 arguments, project shall be able to handle functions with 1 to 5 args, with arbitrary types
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

    // Register the process_values function
    register_tool!(tool_json, fn process_values(a: i32, b: f32, c: bool, d: &str, e: i32) -> String);

    // Simulate LLM output
    let llm_output = r#"
    {
        "arguments": [
            {"e": 100},
            {"a": 42},
            {"c": true},
            {"b": 3.14},
            {"d": "Hello, world!"}
        ],
        "name": "process_values"
    }
    "#;

    //this is where the textual output of upstream ops starts to work
    let llm_parsed: Value = serde_json::from_str(llm_output)?;
    //in this instance, it points to the name of a certain function
    let function_name = llm_parsed["name"].as_str().unwrap();
    //it carries the function's arguments names and the values to run in this instance
    let arguments = llm_parsed["arguments"].as_array().unwrap();

    let registry = TOOL_REGISTRY.lock().unwrap();
    let function = registry.get(function_name).unwrap();

    //the function is invoked with the textual input from the upstream ops
//behind the scence, the textual input are converted according to the logic defined above, the original function is invoked
//outputing results
    let result = function.call(arguments.clone())?;
    println!("Result: {}", result);

    Ok(())
}
