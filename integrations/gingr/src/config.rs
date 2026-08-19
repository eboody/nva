use std::fmt;
use url::Url;

#[derive(Clone, Debug, PartialEq, Eq)]
/// Validated Gingr tenant subdomain, without protocol or host suffix.
pub struct Subdomain(String);

impl Subdomain {
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
    /// Returns the normalized provider or storage string slice.
    pub fn as_str(&self) -> &str {
        self.0.as_str().trim_end_matches('/')
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

#[derive(Clone, Debug, PartialEq, Eq)]
/// Provider label attached to outbound Gingr requests and diagnostics.
pub struct Provider {
    label: Option<String>,
}

impl Provider {}

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
    provider: Provider,
}

impl Client {
    /// Returns the Gingr API base URL used by the client.
    pub fn base_url(&self) -> &BaseUrl {
        &self.base_url
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
            .field("provider", &self.provider)
            .finish()
    }
}

impl fmt::Display for Client {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "Gingr client config {{ base_url: {}, provider: {} }}",
            self.base_url, self.provider
        )
    }
}
