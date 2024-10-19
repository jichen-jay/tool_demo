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
