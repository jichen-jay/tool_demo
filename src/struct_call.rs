use paste::paste;

macro_rules! generate_struct_and_call_method {
    (
        fn $func_name:ident($($arg_name:ident: $arg_type:ty),*) -> $ret_type:ty $body:block
    ) => {
        paste! {
            struct [< Strc_ $func_name >]<F>
            where
                F: Fn($($arg_type),*) -> $ret_type,
            {
                function: F,
            }

            impl<F> [< Strc_ $func_name >]<F>
            where
                F: Fn($($arg_type),*) -> $ret_type,
            {
                fn call(&self, $($arg_name: $arg_type),*) -> $ret_type {
                    (self.function)($($arg_name),*)
                }
            }
        }
    };
}

fn process_values(a: i32, b: f32, c: bool, d: &str, e: i32) -> String {
    format!(
        "Processed: a = {}, b = {}, c = {}, d = {}, e = {}",
        a, b, c, d, e
    )
}

generate_struct_and_call_method! {
    fn process_values(a: i32, b: f32, c: bool, d: &str, e: i32) -> String {
        format!(
            "Processed: a = {}, b = {}, c = {}, d = {}, e = {}",
            a, b, c, d, e
        )
    }
}

fn main() {
    let my_struct = Strc_process_values {
        function: process_values,
    };

    let result = my_struct.call(42, 3.14, true, "example", 100);
    println!("{}", result); // Output: Processed: a = 42, b = 3.14, c = true, d = example, e = 100
}



macro_rules! create_tool_with_function {
    (
        fn $func_name:ident($arg1_name:ident : $arg1_type:ty) -> $ret_type:ty,
        $json_description:expr
    ) => {{
        let arg_name = stringify!($arg1_name).to_string();
        let arg_type = stringify!($arg1_type).to_string();
        let arg_type_clone = arg_type.clone();
        let func = Box::new(move |args: &[&str]| -> Result<String> {
            let parsed_arg = parse_argument(&arg_type, args[0])?;

            let result = $func_name(*parsed_arg.downcast_ref::<$arg1_type>().unwrap());
            Ok(result)
        }) as Box<dyn Fn(&[&str]) -> Result<String> + Send + Sync>;

        Tool {
            name: serde_json::from_str::<Value>($json_description).unwrap()["name"]
                .as_str()
                .unwrap()
                .to_string(),
            function: func,
            arg_names: vec![arg_name],
            arg_types: vec![arg_type_clone],
        }
    }};

    (
        fn $func_name:ident($arg1_name:ident : $arg1_type:ty, $arg2_name:ident : $arg2_type:ty) -> $ret_type:ty,
        $json_description:expr
    ) => {{
        let arg_names = vec![
            stringify!($arg1_name).to_string(),
            stringify!($arg2_name).to_string(),
        ];
        let arg_types = vec![
            stringify!($arg1_type).to_string(),
            stringify!($arg2_type).to_string(),
        ];
        let arg_types_clone = arg_types.clone();

        let func = Box::new(move |args: &[&str]| -> Result<String> {
            if args.len() != 2 {
                return Err(format!("Expected 2 arguments, got {}", args.len()).into());
            }

            let parsed_arg1 = parse_argument(&arg_types[0], args[0])?;
            let parsed_arg2 = parse_argument(&arg_types[1], args[1])?;

            let result = $func_name(
                *parsed_arg1.downcast_ref::<$arg1_type>().unwrap(),
                *parsed_arg2.downcast_ref::<$arg2_type>().unwrap(),
            );
            Ok(result)
        }) as Box<dyn Fn(&[&str]) -> Result<String> + Send + Sync>;

        Tool {
            name: serde_json::from_str::<Value>($json_description).unwrap()["name"]
                .as_str()
                .unwrap()
                .to_string(),
            function: func,
            arg_names: arg_names,
            arg_types: arg_types_clone,
        }
    }};

    (
        fn $func_name:ident($arg1_name:ident : $arg1_type:ty, $arg2_name:ident : $arg2_type:ty, $arg3_name:ident : $arg3_type:ty) -> $ret_type:ty,
        $json_description:expr
    ) => {{
        let arg_names = vec![
            stringify!($arg1_name).to_string(),
            stringify!($arg2_name).to_string(),
            stringify!($arg3_name).to_string(),
        ];
        let arg_types = vec![
            stringify!($arg1_type).to_string(),
            stringify!($arg2_type).to_string(),
            stringify!($arg3_type).to_string(),
        ];
        let arg_types_clone = arg_types.clone();

        let func = Box::new(move |args: &[&str]| -> Result<String> {
            if args.len() != 3 {
                return Err(format!("Expected 3 arguments, got {}", args.len()).into());
            }

            let parsed_arg1 = parse_argument(&arg_types[0], args[0])?;
            let parsed_arg2 = parse_argument(&arg_types[1], args[1])?;
            let parsed_arg3 = parse_argument(&arg_types[2], args[2])?;

            let result = $func_name(
                *parsed_arg1.downcast_ref::<$arg1_type>().unwrap(),
                *parsed_arg2.downcast_ref::<$arg2_type>().unwrap(),
                *parsed_arg3.downcast_ref::<$arg3_type>().unwrap(),
            );
            Ok(result)
        }) as Box<dyn Fn(&[&str]) -> Result<String> + Send + Sync>;

        Tool {
            name: serde_json::from_str::<Value>($json_description).unwrap()["name"]
                .as_str()
                .unwrap()
                .to_string(),
            function: func,
            arg_names: arg_names,
            arg_types: arg_types_clone,
        }
    }};

    (
        fn $func_name:ident($arg1_name:ident : $arg1_type:ty, $arg2_name:ident : $arg2_type:ty, $arg3_name:ident : $arg3_type:ty, $arg4_name:ident : $arg4_type:ty) -> $ret_type:ty,
        $json_description:expr
    ) => {{
        let arg_names = vec![
            stringify!($arg1_name).to_string(),
            stringify!($arg2_name).to_string(),
            stringify!($arg3_name).to_string(),
            stringify!($arg4_name).to_string(),
        ];
        let arg_types = vec![
            stringify!($arg1_type).to_string(),
            stringify!($arg2_type).to_string(),
            stringify!($arg3_type).to_string(),
            stringify!($arg4_type).to_string(),
        ];
        let arg_types_clone = arg_types.clone();

        let func = Box::new(move |args: &[&str]| -> Result<String> {
            if args.len() != 4 {
                return Err(format!("Expected 4 arguments, got {}", args.len()).into());
            }

            let parsed_arg1 = parse_argument(&arg_types[0], args[0])?;
            let parsed_arg2 = parse_argument(&arg_types[1], args[1])?;
            let parsed_arg3 = parse_argument(&arg_types[2], args[2])?;
            let parsed_arg4 = parse_argument(&arg_types[3], args[3])?;

            let result = $func_name(
                *parsed_arg1.downcast_ref::<$arg1_type>().unwrap(),
                *parsed_arg2.downcast_ref::<$arg2_type>().unwrap(),
                *parsed_arg3.downcast_ref::<$arg3_type>().unwrap(),
                *parsed_arg4.downcast_ref::<$arg4_type>().unwrap(),
            );
            Ok(result)
        }) as Box<dyn Fn(&[&str]) -> Result<String> + Send + Sync>;

        Tool {
            name: serde_json::from_str::<Value>($json_description).unwrap()["name"]
                .as_str()
                .unwrap()
                .to_string(),
            function: func,
            arg_names: arg_names,
            arg_types: arg_types_clone,
        }
    }};

    (
        fn $func_name:ident($arg1_name:ident : $arg1_type:ty, $arg2_name:ident : $arg2_type:ty, $arg3_name:ident : $arg3_type:ty, $arg4_name:ident : $arg4_type:ty, $arg5_name:ident : $arg5_type:ty) -> $ret_type:ty,
        $json_description:expr
    ) => {{
        let arg_names = vec![
            stringify!($arg1_name).to_string(),
            stringify!($arg2_name).to_string(),
            stringify!($arg3_name).to_string(),
            stringify!($arg4_name).to_string(),
            stringify!($arg5_name).to_string(),
        ];
        let arg_types = vec![
            stringify!($arg1_type).to_string(),
            stringify!($arg2_type).to_string(),
            stringify!($arg3_type).to_string(),
            stringify!($arg4_type).to_string(),
            stringify!($arg5_type).to_string(),
        ];

        let arg_types_clone = arg_types.clone();

        let func = Box::new(move |args: &[&str]| -> Result<String> {
            if args.len() != 5 {
                return Err(format!("Expected 5 arguments, got {}", args.len()).into());
            }

            let parsed_arg1 = parse_argument(&arg_types[0], args[0])?;
            let parsed_arg2 = parse_argument(&arg_types[1], args[1])?;
            let parsed_arg3 = parse_argument(&arg_types[2], args[2])?;
            let parsed_arg4 = parse_argument(&arg_types[3], args[3])?;
            let parsed_arg5 = parse_argument(&arg_types[4], args[4])?;


            let downcasted_arg1: &$arg1_type = match downcast_argument(&parsed_arg1) {
                Ok(val) => {
                    println!("Successfully downcasted argument 1 to {}", stringify!($arg1_type));
                    val
                },
                Err(e) => {
                    println!("Error downcasting argument 1: {}", e);
                    return Err(e);
                }
            };
    
            let downcasted_arg2: &$arg2_type = match downcast_argument(&parsed_arg2) {
                Ok(val) => {
                    println!("Successfully downcasted argument 2 to {}", stringify!($arg2_type));
                    val
                },
                Err(e) => {
                    println!("Error downcasting argument 2: {}", e);
                    return Err(e);
                }
            };
    
            let downcasted_arg3: &$arg3_type = match downcast_argument(&parsed_arg3) {
                Ok(val) => {
                    println!("Successfully downcasted argument 3 to {}", stringify!($arg3_type));
                    println!("Successfully downcasted argument 3 to {}", val);

                    val
                },
                Err(e) => {
                    println!("Error downcasting argument 3: {}", e);
                    return Err(e);
                }
            };
    
            let downcasted_arg4: &$arg4_type = match downcast_argument(&parsed_arg4) {
                Ok(val) => {
                    println!("Successfully downcasted argument 4 to {}", stringify!($arg4_type));
                    println!("Successfully downcasted argument 4 to {:?}", val);

                    val
                },
                Err(e) => {
                    println!("Error downcasting argument 4: {}", e);
                    return Err(e);
                }
            };
    
            let downcasted_arg5: &$arg5_type = match downcast_argument(&parsed_arg5) {
                Ok(val) => {
                    println!("Successfully downcasted argument 5 to {}", stringify!($arg5_type));
                    val
                },
                Err(e) => {
                    println!("Error downcasting argument 5: {}", e);
                    return Err(e);
                }
            };


            println!(
                "Downcasted arguments: {:?}, {:?}, {:?}, {:?}, {:?}",
                downcasted_arg1, downcasted_arg2, downcasted_arg3, downcasted_arg4, downcasted_arg5
            );
    
            let result = $func_name(
                *downcasted_arg1,
                *downcasted_arg2,
                *downcasted_arg3,
                *downcasted_arg4,
                *downcasted_arg5,
            );
    
            Ok(result)
        }) as Box<dyn Fn(&[&str]) -> Result<String> + Send + Sync>;

        Tool {
            name: serde_json::from_str::<Value>($json_description).unwrap()["name"]
                .as_str()
                .unwrap()
                .to_string(),
            function: func,
            arg_names: arg_names,
            arg_types: arg_types_clone,
        }
    }};
}