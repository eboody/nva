use crate::{config, endpoint, response};
use std::fmt;

/// Result type returned by fallible transport operations.
pub type Result<T> = core::result::Result<T, TransportError>;

#[derive(Debug, thiserror::Error)]
/// Errors raised while building or sending Gingr transport requests.
pub enum TransportError {
    #[error("failed to construct Gingr URL for {method:?} {path}: {source}")]
    /// Gingr URL could not be built or parsed for a specific endpoint request.
    BuildUrl {
        /// HTTP method attached at the failure site.
        method: endpoint::Method,
        /// Provider endpoint path attached at the failure site.
        path: endpoint::Path,
        /// Log-safe request parameters available to diagnostics without exposing secrets.
        redacted_parameters: Vec<(String, String)>,
        /// Underlying URL parser or join error.
        source: url::ParseError,
    },
    #[error("HTTP transport is not implemented for {method:?} {path}")]
    /// Real HTTP transport is not enabled in this build.
    HttpNotImplemented {
        /// HTTP method that would have been sent.
        method: endpoint::Method,
        /// Provider endpoint path that would have been sent.
        path: endpoint::Path,
    },
}

impl TransportError {
    fn build_url(request: &RequestParts, source: url::ParseError) -> Self {
        Self::BuildUrl {
            method: request.method,
            path: request.path,
            redacted_parameters: request.redacted().parameters,
            source,
        }
    }

    fn http_not_implemented(request: &RequestParts) -> Self {
        Self::HttpNotImplemented {
            method: request.method,
            path: request.path,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// HTTP method, path, and parameters for a typed Gingr endpoint request.
pub struct RequestParts {
    method: endpoint::Method,
    path: endpoint::Path,
    parameters: Vec<(String, String)>,
    sensitive_parameter_names: Vec<String>,
}

impl RequestParts {
    /// Starts a builder that makes each provider parameter explicit before request capture.
    pub fn builder() -> RequestPartsBuilder {
        RequestPartsBuilder::default()
    }

    /// Adds the Gingr API key to outbound request parameters.
    pub fn with_api_key(mut self, api_key: &config::ApiKey) -> Self {
        self.parameters
            .push(("key".to_owned(), api_key.expose_for_transport().to_owned()));
        self.sensitive_parameter_names.push("key".to_owned());
        self
    }

    /// Returns the HTTP method required by this Gingr endpoint.
    pub fn method(&self) -> endpoint::Method {
        self.method
    }

    /// Returns the Gingr API path for this endpoint.
    pub fn path(&self) -> endpoint::Path {
        self.path
    }

    /// Returns query parameters that should be sent on the URL.
    pub fn query_pairs(&self) -> &[(String, String)] {
        if self.method == endpoint::Method::Get {
            &self.parameters
        } else {
            &[]
        }
    }

    /// Returns form parameters that should be sent in the request body.
    pub fn form_pairs(&self) -> &[(String, String)] {
        if self.method == endpoint::Method::Post {
            &self.parameters
        } else {
            &[]
        }
    }

    /// Returns a copy safe for logs with sensitive request values removed.
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
        let mut url = base_url
            .join_path(self.path)
            .map_err(|source| TransportError::build_url(self, source))?;
        if self.method == endpoint::Method::Get {
            url.query_pairs_mut().extend_pairs(self.parameters.iter());
        }
        Ok(url)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
/// Builder that classifies Gingr endpoint parameters and redaction rules.
pub struct RequestPartsBuilder {
    method: Option<endpoint::Method>,
    path: Option<endpoint::Path>,
    parameters: Vec<(String, String)>,
    sensitive_parameter_names: Vec<String>,
}

impl RequestPartsBuilder {
    /// Returns the HTTP method required by this Gingr endpoint.
    pub fn method(mut self, method: endpoint::Method) -> Self {
        self.method = Some(method);
        self
    }

    /// Returns the Gingr API path for this endpoint.
    pub fn path(mut self, path: endpoint::Path) -> Self {
        self.path = Some(path);
        self
    }

    /// Adds request parameters before they are separated into query or form fields.
    pub fn parameters(mut self, parameters: Vec<(String, String)>) -> Self {
        self.parameters = parameters;
        self
    }

    /// Marks provider parameters that must be redacted from diagnostics.
    pub fn sensitive_parameter_names(mut self, names: &'static [&'static str]) -> Self {
        self.sensitive_parameter_names = names.iter().map(|name| (*name).to_owned()).collect();
        self
    }

    /// Finalizes the provider request descriptor after required fields are present and wrappers have validated local invariants.
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
/// Log-safe view of a Gingr request with sensitive provider parameters removed.
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

/// Defines the behavior required from a transport participant in the transport workflow.
pub trait Transport {
    /// Fixture callback that receives redacted request parts and returns a raw Gingr response.
    fn send(&self, config: &config::Client, request: RequestParts) -> Result<response::Raw>;
}

#[derive(Clone, Debug, Default)]
/// In-memory Gingr transport used for deterministic tests and fixtures.
pub struct MockTransport;

impl Transport for MockTransport {
    fn send(&self, _config: &config::Client, _request: RequestParts) -> Result<response::Raw> {
        Ok(response::Raw::new(
            response::HttpStatus::OK,
            bytes::Bytes::from_static(b"{}"),
        ))
    }
}

#[derive(Clone, Debug, Default)]
/// Reqwest-backed transport for sending requests to Gingr.
pub struct HttpTransport;

impl Transport for HttpTransport {
    fn send(&self, _config: &config::Client, request: RequestParts) -> Result<response::Raw> {
        Err(TransportError::http_not_implemented(&request))
    }
}

#[derive(Clone, Debug)]
/// Gingr client configuration bundle shared by endpoint builders and transport.
pub struct Client<T = HttpTransport> {
    config: config::Client,
    transport: T,
}

impl Client<HttpTransport> {
    /// Pairs validated Gingr configuration with the transport implementation that will capture or send requests.
    pub fn new(config: config::Client) -> Self {
        Self {
            config,
            transport: HttpTransport,
        }
    }
}

impl<T> Client<T> {
    /// Installs a custom transport implementation, usually for tests.
    pub fn with_transport(config: config::Client, transport: T) -> Self {
        Self { config, transport }
    }

    /// Returns the Gingr client configuration used for requests.
    pub fn config(&self) -> &config::Client {
        &self.config
    }

    /// Returns the raw request parts generated for an endpoint without sending it.
    pub fn capture_request(&self, request: &impl endpoint::Request) -> Result<RequestParts> {
        let request = request.request_parts().with_api_key(self.config.api_key());
        let _ = request.url(self.config.base_url())?;
        Ok(request)
    }

    /// Returns a log-safe representation of the generated Gingr request.
    pub fn redacted_request(&self, request: &impl endpoint::Request) -> Result<RedactedRequest> {
        self.capture_request(request)
            .map(|request| request.redacted())
    }
}

impl<T: Transport> Client<T> {
    /// Sends the typed Gingr request through the configured transport.
    pub fn send(&self, request: &impl endpoint::Request) -> Result<response::Raw> {
        let request = self.capture_request(request)?;
        self.transport.send(&self.config, request)
    }
}
