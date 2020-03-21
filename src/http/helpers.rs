use ::serde::{Deserialize, Serialize};

/// Get a json body with the provided headers and query
#[allow(dead_code)]
pub async fn get_json_with_headers<'a, T, U, H, I, Q>(
    url: U,
    headers: H,
    query: I,
) -> anyhow::Result<T>
where
    for<'de> T: Deserialize<'de>,
    U: AsRef<str>,
    H: IntoIterator<Item = &'a (&'static str, &'a str)>,
    I: IntoIterator<Item = &'a (&'a str, Q)>,
    Q: Serialize + 'a,
{
    get_inner(url, move |mut req| {
        req = req.headers({
            let mut map = reqwest::header::HeaderMap::new();
            for &(k, v) in headers {
                map.insert(k, v.parse()?);
            }
            map
        });
        for (k, v) in query {
            req = req.query(&[(*k, v)])
        }
        Ok(req)
    })
    .await
}

/// Get a json body with the provided query
pub async fn get_json<'a, T, U, I, Q>(url: U, query: I) -> anyhow::Result<T>
where
    for<'de> T: Deserialize<'de>,
    U: AsRef<str>,
    I: IntoIterator<Item = &'a (&'a str, Q)>,
    Q: Serialize + 'a,
{
    get_inner(url, move |mut req| {
        for (k, v) in query {
            req = req.query(&[(*k, v)])
        }
        Ok(req)
    })
    .await
}

async fn get_inner<T>(
    url: impl AsRef<str>,
    func: impl FnOnce(reqwest::RequestBuilder) -> anyhow::Result<reqwest::RequestBuilder>,
) -> anyhow::Result<T>
where
    for<'de> T: Deserialize<'de>,
{
    let req = reqwest::ClientBuilder::new()
        .build()
        .unwrap()
        .get(url.as_ref())
        // this
        // is
        // a
        // shitty
        // api
        .header(
            "User-Agent".parse::<reqwest::header::HeaderName>().unwrap(),
            env!("SHAKEN_USER_AGENT")
                .parse::<reqwest::header::HeaderValue>()
                .unwrap(),
        );

    let req = func(req)?;

    reqwest::Client::new()
        .execute(req.build()?)
        .await?
        .error_for_status()?
        .json()
        .await
        .map_err(Into::into)
}
