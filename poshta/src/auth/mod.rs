use yup_oauth2::{AccessToken, InstalledFlowAuthenticator, InstalledFlowReturnMethod};

pub async fn auth() -> AccessToken {
    let secret = yup_oauth2::parse_application_secret(include_str!("./credentials.json"))
        .expect("must parse credentials.json");

    let auth = InstalledFlowAuthenticator::builder(secret, InstalledFlowReturnMethod::HTTPRedirect)
        .persist_tokens_to_disk("tokencache.json")
        .build()
        .await
        .expect("authenticator must work");

    let scopes = &["https://www.googleapis.com/auth/gmail.readonly"];

    // token(<scopes>) is the one important function of this crate; it does everything to
    // obtain a token that can be sent e.g. as Bearer token.
    match auth.token(scopes).await {
        Ok(token) => token,
        Err(e) => {
            println!("error: {:?}", e);
            panic!("no token");
        }
    }
}
