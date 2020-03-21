use {super::*, crate::*};

#[derive(Debug, Template)]
#[namespace("whatsong")]
enum Response {
    Current {
        title: String,
        duration: u64,
        timestamp: String,
        url: String,
    },
    Previous {
        title: String,
        duration: u64,
        url: String,
    },
    NoSong,
}

pub fn initialize<R>(init: &mut ModuleInit<'_, R>)
where
    R: Responder + Send + 'static,
{
    init.command_map.add("song", current_song);
    init.command_map.add("current", current_song);
    init.command_map.add("previous", previous_song);

    let config = &init.config.whatsong.address;
    init.state.insert(Client::new(config));

    // TODO song list
}

async fn current_song<R>(context: Context<Command>, mut responder: R) -> Result
where
    R: Responder + Send + 'static,
{
    let state = context.state().await;
    let client = state.expect_get::<Client>()?;
    let resp = match match client.current().await {
        Ok(resp) => resp,
        Err(err) => {
            // TODO this should be a different response
            responder.say(&context, &Response::NoSong).await?;
            return Err(err);
        }
    } {
        Some((ts, song)) => Response::Current {
            title: song.title,
            duration: song.duration,
            timestamp: ts.to_string(),
            url: format!("https://youtu.be/{}", song.vid),
        },
        None => Response::NoSong,
    };

    responder.say(&context, &resp).await
}

async fn previous_song<R>(context: Context<Command>, mut responder: R) -> Result
where
    R: Responder + Send + 'static,
{
    let state = context.state().await;
    let client = state.expect_get::<Client>()?;
    let song = match client.previous().await {
        Ok(resp) => resp,
        Err(err) => {
            // TODO this should be a different response
            responder.say(&context, &Response::NoSong).await?;
            return Err(err);
        }
    };
    let resp = Response::Previous {
        title: song.title,
        duration: song.duration,
        url: format!("https://youtu.be/{}", song.vid),
    };

    responder.say(&context, &resp).await
}

#[derive(Debug, serde::Deserialize)]
struct Song {
    id: i64,
    vid: String,
    timestamp: u64,
    duration: u64,
    title: String,
}

#[derive(Clone)]
struct Client {
    address: String,
    client: reqwest::Client,
}

impl Client {
    fn new(address: impl ToString) -> Self {
        Self {
            address: address.to_string(),
            client: reqwest::Client::new(),
        }
    }

    async fn current(&self) -> anyhow::Result<Option<(u64, Song)>> {
        let now = crate::util::timestamp();

        let song: Song = self
            .client
            .get(&format!("{}/current", &self.address))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        Ok(Some(now - song.timestamp)
            .filter(|&ts| song.duration <= now - ts)
            .map(|ts| (ts, song)))
    }

    async fn previous(&self) -> anyhow::Result<Song> {
        self.client
            .get(&format!("{}/previous", &self.address))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
            .map_err(Into::into)
    }

    // TODO song list
}
