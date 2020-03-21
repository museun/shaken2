use serde::Deserialize;
use {super::*, crate::*};

#[derive(Debug, Template)]
#[namespace("crates")]
enum Response<'a> {
    NotExact { name: &'a str, version: &'a str },
    CrateVers { name: &'a str, version: &'a str },
    Links { repo: &'a str, docs: &'a str },
    Description { description: String },
    Unknown { name: &'a str },
    NoCrate,
}

#[derive(Deserialize, Debug)]
struct Crate {
    name: String,
    max_version: String,
    description: Option<String>,
    documentation: Option<String>,
    repository: Option<String>,
    exact_match: bool,
}

pub fn initialize<R>(init: &mut ModuleInit<'_, R>)
where
    R: Responder + Send + 'static,
{
    init.command_map.add("crates", crates);
}

async fn crates<R>(context: Context<Command>, mut responder: R) -> Result
where
    R: Responder + Send + 'static,
{
    let arg = match context.args.tail.get(0) {
        Some(arg) => arg.to_string(),
        None => return responder.reply(&context, &Response::NoCrate).await,
    };

    #[derive(Deserialize)]
    struct Resp {
        crates: Vec<Crate>,
    }

    let data: Resp = crate::http::get_json(
        "https://crates.io/api/v1/crates",
        &[("page", "1"), ("per_page", "1"), ("q", &arg)],
    )
    .await?;

    if data.crates.is_empty() {
        return responder
            .reply(&context, &Response::Unknown { name: &arg })
            .await;
    }

    let (resp, crate_) = match data.crates.iter().find(|d| d.exact_match) {
        Some(crate_) => (
            Response::CrateVers {
                name: &crate_.name,
                version: &crate_.max_version,
            },
            crate_,
        ),
        None => {
            let c = &data.crates[0];
            let resp = Response::NotExact {
                name: &c.name,
                version: &c.max_version,
            };
            (resp, c)
        }
    };

    let mut resp = vec![resp];

    if let Some(ref desc) = crate_.description {
        resp.push(Response::Description {
            description: desc.replace('\n', " "),
        })
    }

    if let (Some(ref docs), Some(ref repo)) = (&crate_.documentation, &crate_.repository) {
        resp.push(Response::Links { repo, docs });
    }

    for resp in resp {
        responder.say(&context, &resp).await?;
    }

    Ok(())
}
