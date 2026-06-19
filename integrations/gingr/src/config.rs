use crate::endpoint;
use secrecy::SecretString;
use std::fmt;
use url::Url;

/// Result type returned by fallible config operations.
pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
/// Errors raised when Gingr account settings are empty, malformed, or unsafe to send to the provider.
pub enum Error {
    #[error("invalid Gingr subdomain: {value}")]
    /// Subdomain was empty or contained characters Gingr tenant hosts cannot use.
    InvalidSubdomain {
        /// Raw subdomain supplied by config or a fixture; keep it visible so setup issues can be corrected.
        value: String,
    },
    #[error("invalid Gingr base URL: {reason}")]
    /// Base URL was not a valid HTTPS Gingr endpoint.
    InvalidBaseUrl {
        /// Reason the URL was rejected before any Gingr request could be built from it.
        reason: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Validated Gingr tenant subdomain, without protocol or host suffix.
pub struct Subdomain(String);

impl Subdomain {
    /// Validates the Gingr tenant segment used to route API calls for one resort account.
    pub fn parse(raw: impl AsRef<str>) -> Result<Self> {
        let value = raw.as_ref();
        let valid = !value.is_empty()
            && value.len() <= 63
            && value
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
            && !value.starts_with('-')
            && !value.ends_with('-');

        if valid {
            Ok(Self(value.to_owned()))
        } else {
            Err(Error::InvalidSubdomain {
                value: value.to_owned(),
            })
        }
    }

    /// Returns the normalized provider or storage string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Subdomain {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Clone, PartialEq, Eq)]
/// Canonical Gingr API base URL with HTTPS and no trailing slash.
pub struct BaseUrl(Url);

impl BaseUrl {
    /// Constructs the canonical Gingr base URL for a tenant subdomain.
    pub fn for_subdomain(subdomain: &Subdomain) -> Self {
        let raw = format!("https://{}.gingrapp.com", subdomain.as_str());
        Self(Url::parse(&raw).expect("constructed Gingr URL is valid"))
    }

    /// Validates the HTTPS Gingr app URL used to reach one provider account.
    pub fn parse(raw: impl AsRef<str>) -> Result<Self> {
        let raw = raw.as_ref();

        let url = Url::parse(raw).map_err(|error| Error::InvalidBaseUrl {
            reason: error.to_string(),
        })?;
        if url.scheme() != "https" {
            return Err(Error::InvalidBaseUrl {
                reason: "Gingr API base URL must use https".to_owned(),
            });
        }
        if url.path() != "/" || url.query().is_some() || url.fragment().is_some() {
            return Err(Error::InvalidBaseUrl {
                reason: "Gingr API base URL must not include path, query, or fragment".to_owned(),
            });
        }
        let host = url.host_str().unwrap_or_default();
        let Some(subdomain) = host.strip_suffix(".gingrapp.com") else {
            return Err(Error::InvalidBaseUrl {
                reason: "host must be a gingrapp.com subdomain".to_owned(),
            });
        };
        Subdomain::parse(subdomain)?;
        Ok(Self(url))
    }

    /// Returns the normalized provider or storage string slice.
    pub fn as_str(&self) -> &str {
        self.0.as_str().trim_end_matches('/')
    }

    pub(crate) fn join_path(
        &self,
        path: endpoint::Path,
    ) -> core::result::Result<Url, url::ParseError> {
        self.0.join(path.as_str().trim_start_matches('/'))
    }
}

impl fmt::Debug for BaseUrl {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("BaseUrl")
            .field(&self.as_str())
            .finish()
    }
}

impl fmt::Display for BaseUrl {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Clone)]
/// Secret Gingr API key kept out of debug output and log-safe request views.
pub struct ApiKey(SecretString);

impl ApiKey {
    /// Wraps the shared Gingr webhook secret without exposing it in debug output.
    pub fn from_secret(raw: impl Into<String>) -> Self {
        Self(SecretString::new(raw.into()))
    }

    pub(crate) fn expose_for_transport(&self) -> &str {
        use secrecy::ExposeSecret;
        self.0.expose_secret()
    }
}

impl fmt::Debug for ApiKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ApiKey(<redacted>)")
    }
}

impl fmt::Display for ApiKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("<redacted>")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Provider label attached to outbound Gingr requests and diagnostics.
pub struct Provider {
    label: Option<String>,
}

impl Provider {
    /// Identifies the generic Gingr provider label.
    pub fn gingr() -> Self {
        Self { label: None }
    }

    /// Identifies a labeled Gingr App provider installation.
    pub fn gingr_app(label: impl Into<String>) -> Self {
        Self {
            label: Some(label.into()),
        }
    }
}

impl fmt::Display for Provider {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.label {
            Some(label) => write!(formatter, "Gingr({label})"),
            None => formatter.write_str("Gingr"),
        }
    }
}

#[derive(Clone)]
/// Gingr client configuration bundle shared by endpoint builders and transport.
pub struct Client {
    base_url: BaseUrl,
    api_key: ApiKey,
    provider: Provider,
}

impl Client {
    /// Bundles the validated Gingr URL and secret key used to capture or send provider requests.
    pub fn new(base_url: BaseUrl, api_key: ApiKey) -> Self {
        Self {
            base_url,
            api_key,
            provider: Provider::gingr(),
        }
    }

    /// Returns the Gingr API base URL used by the client.
    pub fn base_url(&self) -> &BaseUrl {
        &self.base_url
    }

    /// Returns the secret Gingr API key wrapper.
    pub fn api_key(&self) -> &ApiKey {
        &self.api_key
    }

    /// Returns the provider label attached to outbound Gingr requests.
    pub fn provider(&self) -> &Provider {
        &self.provider
    }
}

impl fmt::Debug for Client {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Client")
            .field("base_url", &self.base_url)
            .field("api_key", &"<redacted>")
            .field("provider", &self.provider)
            .finish()
    }
}

impl fmt::Display for Client {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "Gingr client config {{ base_url: {}, api_key: <redacted>, provider: {} }}",
            self.base_url, self.provider
        )
    }
}
