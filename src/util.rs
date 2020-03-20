use crate::{Context, RespondableContext};

#[allow(dead_code)]
pub fn timestamp_ms() -> u64 {
    get_timestamp().as_millis() as _
}

pub fn timestamp() -> u64 {
    get_timestamp().as_secs() as _
}

fn get_timestamp() -> std::time::Duration {
    use std::time::SystemTime;
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
}

pub fn print_backtrace(error: anyhow::Error) {
    for (i, cause) in error.chain().enumerate() {
        if i > 0 {
            eprintln!();
            eprintln!("because");
            eprint!("  ");
        }
        eprintln!("{}", cause);
    }
}

pub fn remove_hashes(input: &str) -> &str {
    let left = input.chars().take_while(|&c| c == '#').count();
    &input[left..]
}

#[allow(dead_code)]
pub fn type_name_val<T>(_ignored: &T) -> &'static str {
    type_name::<T>()
}

pub fn type_name<T>() -> &'static str {
    reduce_type_name(std::any::type_name::<T>())
}

// this is .. unique
pub fn reduce_type_name(mut input: &str) -> &str {
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

pub trait TypeName {
    fn type_name(&self) -> &'static str {
        type_name_val(&self)
    }
}

impl<T> TypeName for T {}

#[derive(Debug)]
pub struct DontCareSigil {}

impl std::fmt::Display for DontCareSigil {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}
impl std::error::Error for DontCareSigil {}

pub trait DontCare<T> {
    fn dont_care(self) -> anyhow::Result<T>;
}

impl<T> DontCare<T> for Option<T> {
    fn dont_care(self) -> anyhow::Result<T> {
        self.ok_or_else(|| DontCareSigil {}.into())
    }
}

pub fn dont_care() -> anyhow::Result<()> {
    Err(DontCareSigil {}.into())
}

pub async fn check_config<T, F, C, I>(context: &mut Context<T>, func: F) -> anyhow::Result<()>
where
    T: RespondableContext,
    C: Clone + Send + Sync + 'static,
    F: Fn(C) -> I,
    I: IntoIterator<Item = String>,
{
    use crate::name_and_id::NameAndId as _;
    let state = context.state().await;
    let config = state.expect_get::<C>()?;
    let config = func(config.clone());
    context.args().room().check_list(config.into_iter())
}
