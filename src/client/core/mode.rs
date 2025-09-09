use crate::constants::USER_AGENT_NAME;

pub trait Mode {
    type Client;
    type Builder;
    type Response;

    fn new(builder: Self::Builder) -> Self;

    fn client() -> Self::Client;
}

macro_rules! impl_mode {
    ($name:ident, {
        Client: $client:ty,
        RequestBuilder: $builder:ty,
        Response: $response:ty
    }) => {
        #[derive(Debug)]
        pub struct $name(pub $builder);

        impl Mode for $name {
            type Client = $client;
            type Builder = $builder;
            type Response = $response;

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
    };
}

impl_mode!(Sync, {
    Client: reqwest::blocking::Client,
    RequestBuilder: reqwest::blocking::RequestBuilder,
    Response: reqwest::blocking::Response
});

impl_mode!(Async, {
    Client: reqwest::Client,
    RequestBuilder: reqwest::RequestBuilder,
    Response: reqwest::Response
});
