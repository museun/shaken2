use anyhow::Context as _;

const TWITCH_OAUTH_TOKEN_ENV_VAR: &str = "SHAKEN_TWITCH_OAUTH_TOKEN";
const TWITCH_CLIENT_ID_ENV_VAR: &str = "SHAKEN_TWITCH_CLIENT_ID";

fn expect_env_var(var: &str) -> anyhow::Result<String> {
    std::env::var(var).with_context(|| format!("`{}` must be set", var))
}

pub struct Secrets {
    pub twitch_oauth_token: String,
    pub twitch_client_id: String,
}

impl Secrets {
    pub fn from_env() -> anyhow::Result<Self> {
        // TODO try all of the secrets before bailing
        // so we can which ones are missing
        let twitch_oauth_token = expect_env_var(TWITCH_OAUTH_TOKEN_ENV_VAR)?;
        let twitch_client_id = expect_env_var(TWITCH_CLIENT_ID_ENV_VAR)?;
        Ok(Self {
            twitch_oauth_token,
            twitch_client_id,
        })
    }
}
