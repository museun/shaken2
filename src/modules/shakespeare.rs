use {super::*, crate::*};

use futures::prelude::*;
use rand::prelude::*;
use tokio::time::{Duration, Instant};

#[derive(Debug, Template)]
#[namespace("shakespeare")]
enum Response<'a> {
    Shakespeare { data: &'a str },
}

pub async fn initialize<R>(init: &mut ModuleInit<'_, R>)
where
    R: Responder + Send + 'static,
{
    let mut config = init.config.clone();
    let Config { shakespeare, .. } = &*config.next().await.expect("initial configuration");

    let config::Shakespeare {
        address,
        chance,
        interval,
        quiet,
        ..
    } = shakespeare;

    // TODO make this change the these settings when the file changes
    let client = Shakespeare::new(
        client::Client::new(address),
        Duration::from_secs(*interval),
        Duration::from_secs(*quiet),
        *chance,
    );

    init.state.insert(client);

    init.command_map.add("speak", command);
    init.passive_list.add(passive);
}

async fn command<R>(mut context: Context<Command>, mut responder: R) -> Result
where
    R: Responder + Send + 'static,
{
    context.args.room().check_list(
        context
            .get_current_config()
            .await?
            .shakespeare
            .whitelist
            .iter(),
    )?;

    let data = {
        let cache = &mut *context.state_mut().await;
        let client = cache.expect_get_mut::<Shakespeare>()?;
        client.trigger().await.dont_care()?
    };
    let resp = Response::Shakespeare { data: &data };
    responder.say(&context, &resp).await
}

async fn passive<R>(mut context: Context<Passive>, mut responder: R) -> Result
where
    R: Responder + Send + 'static,
{
    context.args.room().check_list(
        context
            .get_current_config()
            .await?
            .shakespeare
            .whitelist
            .iter(),
    )?;

    let user = context.get_our_user().await;
    // TODO make this a regex or do some case folding
    let force = context.data().starts_with(&format!("@{}", user.name));

    let data = {
        let cache = &mut *context.state_mut().await;
        let client = cache.expect_get_mut::<Shakespeare>()?;

        if force {
            client.trigger().await.dont_care()?
        } else {
            let mut rng = rand::rngs::SmallRng::from_entropy();
            client.passive(&mut rng).await.dont_care()?
        }
    };

    let resp = Response::Shakespeare { data: &data };
    responder.say(&context, &resp).await
}

pub struct Shakespeare {
    client: client::Client,

    interval: Duration,
    quiet: Duration,
    chance: f32,

    last: Option<Instant>,
}

impl Shakespeare {
    pub fn new(client: client::Client, interval: Duration, quiet: Duration, chance: f32) -> Self {
        Self {
            client,

            interval,
            quiet,
            chance,

            last: None,
        }
    }

    // TODO context

    pub async fn passive<R: ?Sized + Rng>(&mut self, rng: &mut R) -> Option<String> {
        if let Some(last) = self.last.as_mut() {
            let now = Instant::now();
            if now.checked_duration_since(*last)? > self.quiet {
                *last = now;
                return self.generate().await;
            }
        }

        if !rng.gen_bool(self.chance as _) | !self.ensure_less_spam().await {
            return None;
        }

        self.generate().await
    }

    pub async fn trigger(&mut self) -> Option<String> {
        if !self.ensure_less_spam().await {
            return None;
        }

        self.generate().await
    }

    async fn generate(&self) -> Option<String> {
        self.client
            .generate("shakespeare")
            .send()
            .inspect_err(|err| log::error!("cannot generate gibberish: {}", err))
            .await
            .map(|client::types::responses::Generated { data, .. }| data)
            .ok()
    }

    pub async fn next_open_time(&self) -> Option<Duration> {
        let last = self.last.as_ref()?;
        self.interval.checked_sub(Instant::now() - *last)
    }

    async fn ensure_less_spam(&mut self) -> bool {
        match self.next_open_time().await {
            Some(dur) => {
                log::debug!("waiting {:.2?}", dur);
                false
            }
            None => {
                self.last.replace(Instant::now());
                true
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn trigger() {
        use futures::prelude::*;
        use httptest::{mappers::*, responders::*, Expectation, Server};

        let generated = client::types::responses::Generated {
            name: "shakespeare".into(),
            data: "this is a test!".into(),
        };

        let server = Server::run();
        let url = format!("http://{}", server.addr());

        let mut shakespeare = Shakespeare::new(
            client::Client::new(url),
            Duration::from_secs(10),
            Duration::from_secs(30),
            0.15,
        );

        server.expect(
            Expectation::matching(all_of![
                request::method("GET"),
                request::path("/generate/shakespeare"),
            ])
            .respond_with(json_encoded(&generated)),
        );

        tokio::time::pause();

        shakespeare.trigger().await.unwrap();
        tokio::time::advance(Duration::from_millis(300)).await;

        let next = shakespeare.next_open_time().await.unwrap();
        assert!(next < Duration::from_secs(10), "{:.4?}", next);
        assert!(next > Duration::from_secs(1), "{:.4?}", next);

        assert!(shakespeare.trigger().now_or_never().unwrap().is_none());

        tokio::time::advance(Duration::from_secs(30)).await;
        assert!(shakespeare.next_open_time().await.is_none());

        server.expect(
            Expectation::matching(all_of![
                request::method("GET"),
                request::path("/generate/shakespeare"),
            ])
            .respond_with(json_encoded(&generated)),
        );
        shakespeare.trigger().await.unwrap();
    }

    #[tokio::test]
    async fn passive_quiet() {
        use httptest::{mappers::*, responders::*, Expectation, Server};

        let generated = client::types::responses::Generated {
            name: "shakespeare".into(),
            data: "this is a test!".into(),
        };

        let server = Server::run();
        let url = format!("http://{}", server.addr());

        let mut shakespeare = Shakespeare::new(
            client::Client::new(url),
            Duration::from_secs(10),
            Duration::from_secs(30),
            0.15,
        );

        server.expect(
            Expectation::matching(all_of![
                request::method("GET"),
                request::path("/generate/shakespeare"),
            ])
            .respond_with(json_encoded(&generated)),
        );

        tokio::time::pause();

        // set the 'last' time to be now
        shakespeare.trigger().await.unwrap();

        server.expect(
            Expectation::matching(all_of![
                request::method("GET"),
                request::path("/generate/shakespeare"),
            ])
            .respond_with(json_encoded(&generated)),
        );

        let mut rng = rand::rngs::mock::StepRng::new(0, 1);

        tokio::time::advance(Duration::from_secs(31)).await;
        shakespeare.passive(&mut rng).await.unwrap();
        tokio::time::resume();

        assert!(shakespeare.passive(&mut rng).await.is_none());
    }

    #[tokio::test]
    async fn passive_rng() {
        use httptest::{mappers::*, responders::*, Expectation, Server};

        let generated = client::types::responses::Generated {
            name: "shakespeare".into(),
            data: "this is a test!".into(),
        };

        let server = Server::run();
        let url = format!("http://{}", server.addr());

        // chosen by magic
        // pattern should yield [true, false, ..]
        let mut rng = rand::rngs::mock::StepRng::new(1 << 8 | 1 << (8 + 32), 1 << 31);
        let mut shakespeare = Shakespeare::new(
            client::Client::new(url),
            Duration::from_secs(10),
            Duration::from_secs(30),
            0.00000005970,
        );

        server.expect(
            Expectation::matching(all_of![
                request::method("GET"),
                request::path("/generate/shakespeare"),
            ])
            .respond_with(json_encoded(&generated)),
        );

        // true
        shakespeare.passive(&mut rng).await.unwrap();
        // false
        assert!(shakespeare.passive(&mut rng).await.is_none());
        assert!(shakespeare.passive(&mut rng).await.is_none());
    }
}
