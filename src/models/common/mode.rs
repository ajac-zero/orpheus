use crate::constants::USER_AGENT_NAME;

pub trait Mode {
    type Client;
    type Builder;
    type Response;

    fn new(builder: Self::Builder) -> Self;

    fn client() -> Self::Client;
}

#[derive(Debug)]
pub struct Sync(pub reqwest::blocking::RequestBuilder);
impl Mode for Sync {
    type Client = reqwest::blocking::Client;
    type Builder = reqwest::blocking::RequestBuilder;
    type Response = reqwest::blocking::Response;

    fn new(builder: Self::Builder) -> Self {
        Self(builder)
    }

    fn client() -> Self::Client {
        Self::Client::builder()
            .user_agent(USER_AGENT_NAME)
            .use_rustls_tls()
            .build()
            .expect("build request client")
    }
}

#[derive(Debug)]
pub struct Async(pub reqwest::RequestBuilder);
impl Mode for Async {
    type Client = reqwest::Client;
    type Builder = reqwest::RequestBuilder;
    type Response = reqwest::Response;

    fn new(builder: Self::Builder) -> Self {
        Self(builder)
    }

    fn client() -> Self::Client {
        Self::Client::builder()
            .user_agent(USER_AGENT_NAME)
            .use_rustls_tls()
            .build()
            .expect("build request client")
    }
}
