use crate::{config, endpoint, response};
use std::fmt;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to construct Gingr URL: {0}")]
    Url(#[from] url::ParseError),
    #[error("HTTP transport is not implemented for this SDK slice")]
    HttpNotImplemented,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RequestParts {
    method: endpoint::Method,
    path: endpoint::Path,
    parameters: Vec<(String, String)>,
    sensitive_parameter_names: Vec<String>,
}

impl RequestParts {
    pub fn builder() -> RequestPartsBuilder {
        RequestPartsBuilder::default()
    }

    pub fn with_api_key(mut self, api_key: &config::ApiKey) -> Self {
        self.parameters
            .push(("key".to_owned(), api_key.expose_for_transport().to_owned()));
        self.sensitive_parameter_names.push("key".to_owned());
        self
    }

    pub fn method(&self) -> endpoint::Method {
        self.method
    }

    pub fn path(&self) -> endpoint::Path {
        self.path
    }

    pub fn query_pairs(&self) -> &[(String, String)] {
        if self.method == endpoint::Method::Get {
            &self.parameters
        } else {
            &[]
        }
    }

    pub fn form_pairs(&self) -> &[(String, String)] {
        if self.method == endpoint::Method::Post {
            &self.parameters
        } else {
            &[]
        }
    }

    pub fn redacted(&self) -> RedactedRequest {
        RedactedRequest {
            method: self.method,
            path: self.path,
            parameters: self
                .parameters
                .iter()
                .map(|(key, value)| {
                    let rendered = if self
                        .sensitive_parameter_names
                        .iter()
                        .any(|name| name == key)
                    {
                        "<redacted>"
                    } else {
                        value
                    };
                    (key.clone(), rendered.to_owned())
                })
                .collect(),
        }
    }

    fn url(&self, base_url: &config::BaseUrl) -> Result<url::Url> {
        let mut url = base_url.join_path(self.path)?;
        if self.method == endpoint::Method::Get {
            url.query_pairs_mut().extend_pairs(self.parameters.iter());
        }
        Ok(url)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RequestPartsBuilder {
    method: Option<endpoint::Method>,
    path: Option<endpoint::Path>,
    parameters: Vec<(String, String)>,
    sensitive_parameter_names: Vec<String>,
}

impl RequestPartsBuilder {
    pub fn method(mut self, method: endpoint::Method) -> Self {
        self.method = Some(method);
        self
    }

    pub fn path(mut self, path: endpoint::Path) -> Self {
        self.path = Some(path);
        self
    }

    pub fn parameters(mut self, parameters: Vec<(String, String)>) -> Self {
        self.parameters = parameters;
        self
    }

    pub fn sensitive_parameter_names(mut self, names: &'static [&'static str]) -> Self {
        self.sensitive_parameter_names = names.iter().map(|name| (*name).to_owned()).collect();
        self
    }

    pub fn build(self) -> RequestParts {
        RequestParts {
            method: self.method.expect("request method is required"),
            path: self.path.expect("request path is required"),
            parameters: self.parameters,
            sensitive_parameter_names: self.sensitive_parameter_names,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RedactedRequest {
    method: endpoint::Method,
    path: endpoint::Path,
    parameters: Vec<(String, String)>,
}

impl fmt::Display for RedactedRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:?} {}", self.method, self.path)?;
        if !self.parameters.is_empty() {
            let prefix = match self.method {
                endpoint::Method::Get => "?",
                endpoint::Method::Post => " form:",
            };
            let params = self
                .parameters
                .iter()
                .map(|(key, value)| format!("{key}={value}"))
                .collect::<Vec<_>>()
                .join("&");
            write!(formatter, "{prefix}{params}")?;
        }
        Ok(())
    }
}

pub trait Transport {
    fn send(&self, config: &config::ClientConfig, request: RequestParts) -> Result<response::Raw>;
}

#[derive(Clone, Debug, Default)]
pub struct MockTransport;

impl Transport for MockTransport {
    fn send(
        &self,
        _config: &config::ClientConfig,
        _request: RequestParts,
    ) -> Result<response::Raw> {
        Ok(response::Raw::new(
            response::HttpStatus::OK,
            bytes::Bytes::from_static(b"{}"),
        ))
    }
}

#[derive(Clone, Debug, Default)]
pub struct HttpTransport;

impl Transport for HttpTransport {
    fn send(
        &self,
        _config: &config::ClientConfig,
        _request: RequestParts,
    ) -> Result<response::Raw> {
        Err(Error::HttpNotImplemented)
    }
}

#[derive(Clone, Debug)]
pub struct Client<T = HttpTransport> {
    config: config::ClientConfig,
    transport: T,
}

impl Client<HttpTransport> {
    pub fn new(config: config::ClientConfig) -> Self {
        Self {
            config,
            transport: HttpTransport,
        }
    }
}

impl<T> Client<T> {
    pub fn with_transport(config: config::ClientConfig, transport: T) -> Self {
        Self { config, transport }
    }

    pub fn config(&self) -> &config::ClientConfig {
        &self.config
    }

    pub fn capture_request(&self, request: &impl endpoint::Request) -> Result<RequestParts> {
        let request = request.request_parts().with_api_key(self.config.api_key());
        let _ = request.url(self.config.base_url())?;
        Ok(request)
    }

    pub fn redacted_request(&self, request: &impl endpoint::Request) -> Result<RedactedRequest> {
        self.capture_request(request)
            .map(|request| request.redacted())
    }
}

impl<T: Transport> Client<T> {
    pub fn send(&self, request: &impl endpoint::Request) -> Result<response::Raw> {
        let request = self.capture_request(request)?;
        self.transport.send(&self.config, request)
    }
}
