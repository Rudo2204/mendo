use anyhow::{anyhow, Result};
use log::{debug, error, info};
use oauth2::{AuthorizationCode, CsrfToken};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use url::Url;

use crate::util::MendoConfig;

pub fn auth(cfg: &mut MendoConfig) -> Result<String> {
    let client_id = &cfg.id.to_string();
    let client_secret = &cfg.secret;
    let redirect_uri = &cfg.url.to_string();
    let state = CsrfToken::new_random().secret().to_string();

    let url = Url::parse_with_params(
        "https://anilist.co/api/v2/oauth/authorize",
        &[
            ("client_id", client_id),
            ("redirect_uri", redirect_uri),
            ("response_type", &"code".to_string()),
            ("state", &state),
        ],
    )?;

    let mut post_json = HashMap::new();
    post_json.insert("grant_type", "authorization_code");
    post_json.insert("client_id", &client_id);
    post_json.insert("client_secret", &client_secret);
    post_json.insert("redirect_uri", &redirect_uri);

    debug!("Setup ready. Attempting to open browser...");
    println!("Opening browser to authorize...");

    open::that(url.to_string())?;

    //Naive way to implement the redirect server
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    if let Ok((mut stream, _)) = listener.accept() {
        debug!("OK! Found stream!");

        let code;
        {
            let mut reader = BufReader::new(&mut stream);
            let mut request_line = String::new();

            reader.read_line(&mut request_line)?;
            let redirect_url = request_line
                .split_whitespace()
                .nth(1)
                .expect("Safe because of how anilist defines redirect URI");
            let url = Url::parse(&format!("http://localhost{}", redirect_url))?;

            let code_pair = url
                .query_pairs()
                .find(|pair| {
                    let &(ref key, _) = pair;
                    key == "code"
                })
                .expect("Something went wrong in the authorization process!");

            let (_, value) = code_pair;
            code = AuthorizationCode::new(value.into_owned());
            // It also returns urlState, but we don't care about it.
        }

        post_json.insert("code", &code.secret());

        let message = "Finished. Return to your terminal!";
        let response = format!(
            "HTTP/1.1 200 OK\r\ncontent-length: {}\r\n\r\n{}",
            message.len(),
            message
        );
        stream.write_all(response.as_bytes())?;

        debug!("Anilist returned the following code:\n{}\n", code.secret());
        debug!("Now will exchange it for access token...");

        let client = reqwest::blocking::Client::new();
        let token_res = client
            .post("https://anilist.co/api/v2/oauth/token")
            .header("Accept", "application/json")
            .json(&post_json)
            .send()?
            .text()?;

        debug!("Anilist returned the following token:\n{}\n", token_res);
        info!("Successfully authenticated the user!");
        Ok(token_res)
    } else {
        error!("Could not find stream !?");
        Err(anyhow!("Something went wrong trying to authorize!"))
    }
}
