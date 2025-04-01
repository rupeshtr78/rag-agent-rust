pub mod constants;
use hyper_rustls::HttpsConnector;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client as LegacyClient;
use http_body_util::Full;
use hyper::body::Bytes;
use rustls::crypto::ring::default_provider;
use hyper_util::rt::TokioExecutor;
use log::debug;

pub type HttpsClient = LegacyClient<HttpsConnector<HttpConnector>, Full<Bytes>>;

pub fn get_https_client() -> anyhow::Result<HttpsClient> {
    // Install the crypto provider required by rustls
    match default_provider().install_default() {
        anyhow::Result::Ok(_) => debug!("Crypto provider installed successfully"),
        Err(e) => {
            return Err(anyhow::anyhow!(
                "Failed to install crypto provider: {:?}",
                e
            ));
        }
    }

    // Create an HTTPS connector with native roots
    let https = hyper_rustls::HttpsConnectorBuilder::new()
        .with_native_roots()?
        .https_or_http()
        .enable_http1()
        .build();

    // Build the hyper client from the HTTPS connector
    let client: HttpsClient = LegacyClient::builder(TokioExecutor::new()).build(https);
    anyhow::Ok(client)
}