pub mod constants;
use hyper_rustls::HttpsConnector;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client as LegacyClient;
use http_body_util::Full;
use hyper::body::Bytes;
pub type HttpsClient = LegacyClient<HttpsConnector<HttpConnector>, Full<Bytes>>;