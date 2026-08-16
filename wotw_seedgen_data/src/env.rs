use std::{env, fmt::Display, str::FromStr};

pub fn env_or<T>(key: &str, default: T) -> T
where
    T: FromStr,
    T::Err: Display,
{
    env::var_os(key)
        .and_then(|s| {
            s.into_string()
                .map_err(|err| eprintln!("env {key}={err:?} was not valid unicode"))
                .and_then(|s| {
                    s.parse()
                        .map_err(|err| eprintln!("failed to parse env {key}={s}: {err}"))
                })
                .ok()
        })
        .unwrap_or(default)
}
