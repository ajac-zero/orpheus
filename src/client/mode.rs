use std::collections::HashMap;

use reqwest::header::HeaderMap;

use crate::constants::USER_AGENT_NAME;

pub trait Mode {
    type Client: Clone + std::fmt::Debug;
    type Response;

    fn new(client: Self::Client) -> Self;

    fn client(headers: HashMap<String, String>) -> Self::Client;
}

macro_rules! impl_mode {
    ($name:ident, {
        Client: $client:ty,
        Response: $response:ty
    }) => {
        #[derive(Debug)]
        pub struct $name {
            pub client: $client,
        }

        impl Mode for $name {
            type Client = $client;
            type Response = $response;

            fn new(client: Self::Client) -> Self {
                Self { client }
            }

            fn client(headers: HashMap<String, String>) -> Self::Client {
                Self::Client::builder()
                    .user_agent(USER_AGENT_NAME)
                    .use_rustls_tls()
                    .default_headers({
                        HeaderMap::from_iter(
                            headers
                                .into_iter()
                                .map(|(k, v)| (k.parse().unwrap(), v.parse().unwrap())),
                        )
                    })
                    .build()
                    .expect("build request client")
            }
        }
    };
}

impl_mode!(Sync, {
    Client: reqwest::blocking::Client,
    Response: reqwest::blocking::Response
});

impl_mode!(Async, {
    Client: reqwest::Client,
    Response: reqwest::Response
});
