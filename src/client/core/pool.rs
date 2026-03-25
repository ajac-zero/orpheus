use std::{collections::HashMap, sync::Arc};

use deadpool::managed;
use hyper::client::conn::http2;
use hyper_util::rt::{TokioExecutor, TokioIo};
use rustls::{ClientConfig, RootCertStore};
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;

use crate::{
    Error, Result,
    client::{
        handler::Handler,
        mode::{Async, Mode, Sync},
    },
};

pub(crate) struct Pool<M: Mode> {
    inner: managed::Pool<Manager<M>>,
    mode: M,
}

impl<M: Mode> Pool<M> {
    pub fn new(url: url::Url, headers: HashMap<String, String>) -> Self {
        let mode = M::new();
        let mgr = Manager::new(url, headers, mode.clone());
        let inner = managed::Pool::builder(mgr).build().unwrap();
        Self { inner, mode }
    }
}

impl Pool<Sync> {
    pub fn get(&self) -> Result<managed::Object<Manager<Sync>>> {
        Ok(self
            .mode
            .rt
            .block_on(async { self.inner.get().await.unwrap() }))
    }
}

impl Pool<Async> {
    pub async fn get(&self) -> Result<managed::Object<Manager<Async>>> {
        Ok(self.inner.get().await.unwrap())
    }
}

pub struct Manager<M: Mode> {
    url: url::Url,
    headers: HashMap<String, String>,
    exec: TokioExecutor,
    connector: Option<TlsConnector>,
    mode: M,
}

impl<M: Mode> Manager<M> {
    fn new(url: url::Url, headers: HashMap<String, String>, mode: M) -> Self {
        let exec = TokioExecutor::new();

        let connector = if url.scheme() == "https" {
            let mut root_store = RootCertStore::empty();
            root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

            let provider = rustls::crypto::aws_lc_rs::default_provider();
            let mut config = ClientConfig::builder_with_provider(Arc::new(provider))
                .with_safe_default_protocol_versions()
                .unwrap()
                .with_root_certificates(root_store)
                .with_no_client_auth();
            config.alpn_protocols = vec![b"h2".to_vec()];

            Some(TlsConnector::from(Arc::new(config)))
        } else {
            None
        };

        Self {
            url,
            headers,
            exec,
            connector,
            mode,
        }
    }
}

impl<M: Mode> managed::Manager for Manager<M> {
    type Type = Handler<M>;
    type Error = Error;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        let host = self.url.host_str().unwrap();
        let port = self.url.port_or_known_default().unwrap();

        let tcp = TcpStream::connect((host, port)).await?;
        let exec = self.exec.clone();

        let (sender, handle) = if let Some(connector) = self.connector.as_ref() {
            let domain = host.to_string().try_into().unwrap();
            let tls_stream = connector.connect(domain, tcp).await?;
            let io = TokioIo::new(tls_stream);

            let (sender, conn) = http2::handshake(exec, io).await.unwrap();

            let handle = tokio::spawn(async move {
                if let Err(e) = conn.await {
                    eprintln!("HTTP/2 connection ended: {:?}", e);
                }
            });

            (sender, handle)
        } else {
            let io = TokioIo::new(tcp);

            let (sender, conn) = http2::handshake(exec, io).await.unwrap();

            let handle = tokio::spawn(async move {
                if let Err(e) = conn.await {
                    eprintln!("HTTP/2 connection ended: {:?}", e);
                }
            });

            (sender, handle)
        };

        Ok(Self::Type {
            url: self.url.clone(),
            headers: self.headers.clone(),
            sender,
            _handle: handle,
            mode: self.mode.clone(),
        })
    }

    async fn recycle(
        &self,
        handler: &mut Self::Type,
        _: &managed::Metrics,
    ) -> managed::RecycleResult<Self::Error> {
        if handler.sender.is_ready() {
            Ok(())
        } else {
            Err(managed::RecycleError::message("Connection is not ready"))
        }
    }
}

impl<M: Mode> std::fmt::Debug for Pool<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Pool").finish()
    }
}
