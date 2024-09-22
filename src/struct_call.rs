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
