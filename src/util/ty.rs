/// Get a short representation of the provided type
pub fn type_name<T>() -> &'static str {
    reduce_type_name(std::any::type_name::<T>())
}

pub fn type_name_of<T>(_ignored: &T) -> &'static str {
    std::any::type_name::<T>()
}

/// Tries to reduce a complex type name down to its base type
pub fn reduce_type_name(mut input: &str) -> &str {
    // this is .. totally not something you should do
    fn trim_type(input: &str) -> &str {
        let mut n = input.len();
        let left = input
            .chars()
            .take_while(|&c| {
                if c == '<' {
                    n -= 1;
                }
                !c.is_ascii_uppercase()
            })
            .count();
        &input[left..n]
    }

    let original = input;
    loop {
        let start = input.len();
        input = trim_type(input);
        if input.contains('<') {
            input = trim_type(&input[1..]);
        }
        match input.len() {
            0 => break original,
            d if d == start => break input,
            _ => {}
        }
    }
}

/// Get a reduced for this current time
pub trait TypeName {
    fn type_name(&self) -> &'static str {
        #[allow(dead_code)]
        fn ty<T>(_ignored: &T) -> &'static str {
            reduce_type_name(type_name::<T>())
        }
        ty(&self)
    }
}

impl<T> TypeName for T {}
