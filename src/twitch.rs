use crate::serde_util;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct Client {
    client: reqwest::Client,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Users {
    #[serde(skip_deserializing)]
    pub room: String,
    #[serde(skip_deserializing)]
    pub chatter_count: usize,
    pub broadcaster: Vec<String>,
    pub vips: Vec<String>,
    pub moderators: Vec<String>,
    pub staff: Vec<String>,
    pub admins: Vec<String>,
    pub global_mods: Vec<String>,
    pub viewers: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Stream {
    #[serde(deserialize_with = "serde_util::from_str")]
    pub id: u64,
    #[serde(deserialize_with = "serde_util::from_str")]
    pub user_id: u64,
    pub user_name: String,
    #[serde(deserialize_with = "serde_util::from_str")]
    pub game_id: u64,
    #[serde(rename = "type")]
    pub type_: Option<String>, // TODO enum
    pub title: String,
    pub viewer_count: u64,
    #[serde(deserialize_with = "serde_util::assume_utc_date_time")]
    pub started_at: time::OffsetDateTime,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct User {
    #[serde(deserialize_with = "serde_util::from_str")]
    pub id: u64,
    pub login: String,
    pub display_name: String,
}

impl Client {
    const BASE_URI: &'static str = "https://api.twitch.tv/helix";

    pub fn new(client_id: &str) -> Self {
        let client = reqwest::ClientBuilder::new()
            .user_agent(env!("SHAKEN_USER_AGENT"))
            .default_headers({
                let mut map = reqwest::header::HeaderMap::new();
                map.insert("Client-ID", client_id.parse().unwrap());
                map
            })
            .build()
            .unwrap();
        Self { client }
    }

    pub async fn get_streams<I>(&self, user_logins: I) -> anyhow::Result<Vec<Stream>>
    where
        I: IntoIterator,
        I::Item: serde::Serialize,
    {
        #[derive(Deserialize)]
        struct Data {
            data: Vec<Stream>,
        }

        self.get_response::<Data, _, _>("streams", std::iter::repeat("user_login").zip(user_logins))
            .await
            .map(|data| data.data)
    }

    #[allow(dead_code)]
    pub async fn get_streams_from_id<I>(&self, user_ids: I) -> anyhow::Result<Vec<Stream>>
    where
        I: IntoIterator,
        I::Item: serde::Serialize,
    {
        #[derive(Deserialize)]
        struct Data {
            data: Vec<Stream>,
        }

        self.get_response::<Data, _, _>("streams", std::iter::repeat("user_id").zip(user_ids))
            .await
            .map(|data| data.data)
    }

    #[allow(dead_code)]
    pub async fn get_users<I>(&self, user_logins: I) -> anyhow::Result<Vec<User>>
    where
        I: IntoIterator,
        I::Item: serde::Serialize,
    {
        #[derive(Deserialize)]
        struct Data {
            data: Vec<User>,
        }

        self.get_response::<Data, _, _>("users", std::iter::repeat("login").zip(user_logins))
            .await
            .map(|data| data.data)
    }

    #[allow(dead_code)]
    pub async fn get_users_from_id<I>(&self, user_ids: I) -> anyhow::Result<Vec<User>>
    where
        I: IntoIterator,
        I::Item: serde::Serialize,
    {
        #[derive(Deserialize)]
        struct Data {
            data: Vec<User>,
        }

        self.get_response::<Data, _, _>("users", std::iter::repeat("id").zip(user_ids))
            .await
            .map(|data| data.data)
    }

    #[allow(dead_code)]
    pub async fn get_names_for(&self, room: &str) -> anyhow::Result<Users> {
        #[derive(Deserialize)]
        struct Data {
            chatter_count: usize,
            chatters: Users,
        }

        let req = self
            .client
            .get(&format!(
                "https://tmi.twitch.tv/group/user/{}/chatters",
                room
            ))
            .build()?;

        self.client
            .execute(req)
            .await?
            .error_for_status()?
            .json()
            .await
            .map(|data: Data| Users {
                room: room.to_string(),
                chatter_count: data.chatter_count,
                ..data.chatters
            })
            .map_err(Into::into)
    }

    async fn get_response<'a, T, M, V>(&self, ep: &str, map: M) -> anyhow::Result<T>
    where
        for<'de> T: serde::Deserialize<'de>,
        M: IntoIterator<Item = (&'a str, V)>,
        V: serde::Serialize,
    {
        let mut req = self.client.get(&format!("{}/{}", Self::BASE_URI, ep));
        for (key, val) in map {
            req = req.query(&[(key, val)]);
        }
        self.client
            .execute(req.build()?)
            .await?
            .error_for_status()?
            .json()
            .await
            .map_err(Into::into)
    }
}
