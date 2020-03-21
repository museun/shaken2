use anyhow::Context as _;
use std::collections::HashMap;

pub const TWITCH_OAUTH_TOKEN: &str = "SHAKEN_TWITCH_OAUTH_TOKEN";
pub const TWITCH_CLIENT_ID: &str = "SHAKEN_TWITCH_CLIENT_ID";

#[derive(Default)]
pub struct Secrets {
    map: HashMap<String, String>,
}

impl Secrets {
    pub fn get(&self, secret: &str) -> anyhow::Result<&String> {
        self.map
            .get(secret)
            .ok_or_else(|| anyhow::anyhow!("secret `{}` was not found", secret))
    }

    pub fn take(&mut self, secret: &str) -> anyhow::Result<String> {
        self.map
            .remove(secret)
            .ok_or_else(|| anyhow::anyhow!("secret `{}` was not found", secret))
    }

    pub fn from_env() -> anyhow::Result<Self> {
        const DESIRED: &[&str] = &[TWITCH_OAUTH_TOKEN, TWITCH_CLIENT_ID];

        fn expect_env_var(var: &str) -> anyhow::Result<String> {
            std::env::var(var).with_context(|| format!("`{}` must be set", var))
        }

        let mut map: HashMap<_, Option<_>> = HashMap::default();
        for &desired in DESIRED {
            let item = match expect_env_var(desired) {
                Err(err) => {
                    eprintln!("Error: {}", err);
                    None
                }
                Ok(item) => Some(item),
            };

            map.insert(desired.to_string(), item);
        }

        if map.values().any(|s| s.is_none()) {
            anyhow::bail!("some of the secrets weren't loaded");
        }

        let map = map
            .into_iter()
            .filter_map(|(k, v)| (k, v?).into())
            .collect();

        Ok(Self { map })
    }
}
