use crate::constants::USER_AGENT_NAME;

pub trait Mode {
    type Client: Clone + std::fmt::Debug;
    type Response;

    fn new(client: Self::Client) -> Self;

    fn client() -> Self::Client;
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

            fn client() -> Self::Client {
                Self::Client::builder()
                    .user_agent(USER_AGENT_NAME)
                    .use_rustls_tls()
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
