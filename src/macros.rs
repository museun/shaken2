#[macro_export]
macro_rules! maybe {
    ($expr:expr) => {{
        match $expr {
            Some(d) => d,
            None => {
                use crate::util::TypeName as _;
                let name = $expr.type_name();
                maybe!(@LOG &name, "")
            }
        }
    }};

    ($expr:expr, $fmt:expr) => {{
        match $expr {
            Some(d) => d,
            None => {
                use crate::util::TypeName as _;
                let name = $expr.type_name();
                maybe!(@LOG &name, $expr.type_name(), $fmt)
            }
        }
    }};

    ($expr:expr, $fmt:expr, $($args:expr),* $(,)?) => {{
        match $expr {
            Some(d) => d,
            None => {
                use crate::util::TypeName as _;
                let name = $expr.type_name();
                maybe!(@LOG &name, format_args!($fmt, $($args),*))
            }
        }
    }};

    (@LOG $ty:expr, $msg:expr) => {{
        match $msg.is_empty() {
            true => {
                log::trace!(
                    "expected a '{}' @ {}:{}:{} ({})",
                    $ty,
                    file!(),
                    line!(),
                    column!(),
                    module_path!(),
                );
            },
            false => {
                log::trace!(
                    "expected a '{}': {} @ {}:{}:{} ({})",
                    $ty,
                    $msg,
                    file!(),
                    line!(),
                    column!(),
                    module_path!(),
                );
            }
        }
        return Ok(());
    }}
}
