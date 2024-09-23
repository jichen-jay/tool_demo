use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

type Result<T> = std::result::Result<T, Box<dyn Error>>;
//intend to use this macro to extract the arg names and arg types into 2 array
//intend to use it like a function, returning (Vec<String>, Vec<String>)

macro_rules! extract_args_and_types {
    (
        fn $func_name:ident($($arg_name:ident : $arg_type:ty),*) -> $ret_type:ty
    ) => {
        let arg_names: &[&str] = &[$(stringify!($arg_name)),*];
        let arg_types: &[&str] = &[$(stringify!($arg_type)),*];
        (arg_names, arg_types)
    };
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
