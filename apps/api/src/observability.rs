use std::{
    collections::BTreeMap,
    fmt,
    sync::{Arc, Mutex},
    time::Duration,
};

use axum::http::Uri;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_otlp::WithExportConfig as _;
use opentelemetry_sdk::{Resource, trace::SdkTracerProvider};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt as _, util::SubscriberInitExt as _};

const DURATION_BUCKETS_SECONDS: [f64; 8] = [0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Runtime telemetry posture selected for the API process.
pub enum TelemetryMode {
    /// Local-only telemetry that makes no durability, dashboard, or alerting claim.
    Local,
    /// Production telemetry backed by explicitly configured export and operations references.
    Production,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Truthful readiness claims for each durable observability capability.
pub struct ObservabilityReadiness {
    /// Durable trace-export readiness vocabulary.
    pub durable_traces: &'static str,
    /// Production metric-scraping readiness vocabulary.
    pub production_metrics: &'static str,
    /// Provisioned dashboard readiness vocabulary.
    pub dashboard: &'static str,
    /// Configured alert-policy readiness vocabulary.
    pub alerting: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
/// Validated observability mode and the evidence required by that mode.
pub enum ObservabilityConfig {
    #[default]
    /// Local mode, which intentionally claims no durable production telemetry.
    Local,
    /// Production mode containing validated exporter and operator-facing references.
    Production(ProductionObservability),
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Validated production telemetry configuration required before production startup.
pub struct ProductionObservability {
    otlp_endpoint: String,
    dashboard_url: String,
    alert_policy: String,
}

impl ObservabilityConfig {
    /// Constructs the explicit local-only telemetry posture.
    pub const fn local() -> Self {
        Self::Local
    }

    /// Validates telemetry configuration from an arbitrary environment-like key/value source.
    pub fn try_from_pairs<I, K, V>(pairs: I) -> Result<Self, ObservabilityConfigError>
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        let values = pairs
            .into_iter()
            .map(|(key, value)| (key.into(), value.into()))
            .collect::<BTreeMap<_, _>>();
        match values
            .get("PET_RESORT_TELEMETRY_MODE")
            .map(String::as_str)
            .unwrap_or("local")
        {
            "local" => Ok(Self::Local),
            "production" => {
                let otlp_endpoint = required(
                    &values,
                    "OTEL_EXPORTER_OTLP_ENDPOINT",
                    ObservabilityConfigError::MissingOtlpEndpoint,
                )?;
                validate_http_uri(&otlp_endpoint, "OTEL_EXPORTER_OTLP_ENDPOINT")?;
                let dashboard_url = required(
                    &values,
                    "PET_RESORT_OBSERVABILITY_DASHBOARD_URL",
                    ObservabilityConfigError::MissingDashboardUrl,
                )?;
                validate_http_uri(&dashboard_url, "PET_RESORT_OBSERVABILITY_DASHBOARD_URL")?;
                let alert_policy = required(
                    &values,
                    "PET_RESORT_OBSERVABILITY_ALERT_POLICY",
                    ObservabilityConfigError::MissingAlertPolicy,
                )?;
                Ok(Self::Production(ProductionObservability {
                    otlp_endpoint,
                    dashboard_url,
                    alert_policy,
                }))
            }
            mode => Err(ObservabilityConfigError::InvalidMode(mode.to_owned())),
        }
    }

    /// Validates telemetry configuration from the current process environment.
    pub fn from_process_env() -> Result<Self, ObservabilityConfigError> {
        Self::try_from_pairs(std::env::vars())
    }

    /// Returns the selected telemetry mode without exposing configuration details.
    pub const fn mode(&self) -> TelemetryMode {
        match self {
            Self::Local => TelemetryMode::Local,
            Self::Production(_) => TelemetryMode::Production,
        }
    }

    /// Returns truthful readiness claims derived from the validated mode.
    pub const fn readiness(&self) -> ObservabilityReadiness {
        match self {
            Self::Local => ObservabilityReadiness {
                durable_traces: "not_configured",
                production_metrics: "not_configured",
                dashboard: "not_configured",
                alerting: "not_configured",
            },
            Self::Production(_) => ObservabilityReadiness {
                durable_traces: "configured_otlp",
                production_metrics: "configured_prometheus",
                dashboard: "configured_grafana",
                alerting: "configured_prometheus_rules",
            },
        }
    }

    /// Returns production exporter and operator references only in production mode.
    pub fn production(&self) -> Option<&ProductionObservability> {
        match self {
            Self::Local => None,
            Self::Production(config) => Some(config),
        }
    }
}

impl ProductionObservability {
    /// Returns the validated OTLP collector endpoint.
    pub fn otlp_endpoint(&self) -> &str {
        &self.otlp_endpoint
    }

    /// Returns the configured operator-facing dashboard URL.
    pub fn dashboard_url(&self) -> &str {
        &self.dashboard_url
    }

    /// Returns the configured alert-policy document reference.
    pub fn alert_policy(&self) -> &str {
        &self.alert_policy
    }
}

fn required(
    values: &BTreeMap<String, String>,
    key: &'static str,
    missing: ObservabilityConfigError,
) -> Result<String, ObservabilityConfigError> {
    values
        .get(key)
        .filter(|value| !value.trim().is_empty())
        .cloned()
        .ok_or(missing)
}

fn validate_http_uri(value: &str, field: &'static str) -> Result<(), ObservabilityConfigError> {
    let uri = value
        .parse::<Uri>()
        .map_err(|_| ObservabilityConfigError::InvalidUri { field })?;
    if !matches!(uri.scheme_str(), Some("http" | "https")) || uri.authority().is_none() {
        return Err(ObservabilityConfigError::InvalidUri { field });
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Reasons observability configuration cannot be promoted into a runtime posture.
pub enum ObservabilityConfigError {
    /// Production mode omitted the OTLP collector endpoint.
    MissingOtlpEndpoint,
    /// Production mode omitted the operator dashboard URL.
    MissingDashboardUrl,
    /// Production mode omitted the alert-policy reference.
    MissingAlertPolicy,
    /// The telemetry mode was neither `local` nor `production`.
    InvalidMode(String),
    /// An endpoint/reference expected to be an absolute HTTP URI was invalid.
    InvalidUri {
        /// Environment field containing the invalid URI.
        field: &'static str,
    },
}

impl fmt::Display for ObservabilityConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingOtlpEndpoint => {
                formatter.write_str("production telemetry requires OTEL_EXPORTER_OTLP_ENDPOINT")
            }
            Self::MissingDashboardUrl => formatter
                .write_str("production telemetry requires PET_RESORT_OBSERVABILITY_DASHBOARD_URL"),
            Self::MissingAlertPolicy => formatter
                .write_str("production telemetry requires PET_RESORT_OBSERVABILITY_ALERT_POLICY"),
            Self::InvalidMode(mode) => write!(formatter, "unsupported telemetry mode {mode}"),
            Self::InvalidUri { field } => write!(formatter, "{field} must be an absolute HTTP URI"),
        }
    }
}

impl std::error::Error for ObservabilityConfigError {}

#[derive(Clone)]
/// Shared API observability state containing validated config and bounded request metrics.
pub struct ObservabilityRuntime {
    config: Arc<ObservabilityConfig>,
    metrics: Arc<Mutex<BTreeMap<RequestSeries, RequestMetric>>>,
}

impl ObservabilityRuntime {
    /// Constructs runtime telemetry state from already validated configuration.
    pub fn new(config: ObservabilityConfig) -> Self {
        Self {
            config: Arc::new(config),
            metrics: Arc::new(Mutex::new(BTreeMap::new())),
        }
    }

    /// Returns the validated configuration used by readiness and tracing installation.
    pub fn config(&self) -> &ObservabilityConfig {
        &self.config
    }

    /// Installs structured logging and, in production mode, an OTLP span exporter.
    pub fn install_tracing(&self) -> Result<TracingGuard, TracingInstallError> {
        let filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("info,tower_http=info"));
        match self.config.as_ref() {
            ObservabilityConfig::Local => {
                tracing_subscriber::registry()
                    .with(filter)
                    .with(tracing_subscriber::fmt::layer().json())
                    .try_init()
                    .map_err(|error| TracingInstallError(error.to_string()))?;
                Ok(TracingGuard { provider: None })
            }
            ObservabilityConfig::Production(config) => {
                let exporter = opentelemetry_otlp::SpanExporter::builder()
                    .with_tonic()
                    .with_endpoint(config.otlp_endpoint())
                    .build()
                    .map_err(|error| TracingInstallError(error.to_string()))?;
                let provider = SdkTracerProvider::builder()
                    .with_batch_exporter(exporter)
                    .with_resource(
                        Resource::builder()
                            .with_service_name("pet-resort-api")
                            .build(),
                    )
                    .build();
                let tracer = provider.tracer("pet-resort-api");
                tracing_subscriber::registry()
                    .with(filter)
                    .with(tracing_opentelemetry::layer().with_tracer(tracer))
                    .with(tracing_subscriber::fmt::layer().json())
                    .try_init()
                    .map_err(|error| TracingInstallError(error.to_string()))?;
                Ok(TracingGuard {
                    provider: Some(provider),
                })
            }
        }
    }

    pub(crate) fn record_request(
        &self,
        method: &str,
        route: &str,
        status: u16,
        duration: Duration,
    ) {
        let series = RequestSeries {
            method: bounded_method(method),
            route: bounded_route(route),
            status_class: status_class(status),
        };
        let mut metrics = self
            .metrics
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        metrics.entry(series).or_default().record(duration);
    }

    /// Renders bounded in-process request aggregates in Prometheus exposition format.
    pub fn render_prometheus(&self) -> String {
        let metrics = self
            .metrics
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut output = String::from(
            "# HELP pet_resort_api_requests_total Total HTTP requests by bounded route and status class.\n\
# TYPE pet_resort_api_requests_total counter\n\
# HELP pet_resort_api_request_duration_seconds Request duration by bounded route and status class.\n\
# TYPE pet_resort_api_request_duration_seconds histogram\n",
        );
        for (series, metric) in metrics.iter() {
            let labels = series.labels();
            output.push_str(&format!(
                "pet_resort_api_requests_total{{{labels}}} {}\n",
                metric.count
            ));
            for (bound, count) in DURATION_BUCKETS_SECONDS.iter().zip(metric.buckets) {
                output.push_str(&format!(
                    "pet_resort_api_request_duration_seconds_bucket{{{labels},le=\"{bound}\"}} {count}\n"
                ));
            }
            output.push_str(&format!(
                "pet_resort_api_request_duration_seconds_bucket{{{labels},le=\"+Inf\"}} {}\n",
                metric.count
            ));
            output.push_str(&format!(
                "pet_resort_api_request_duration_seconds_sum{{{labels}}} {}\n",
                metric.duration_sum_seconds
            ));
            output.push_str(&format!(
                "pet_resort_api_request_duration_seconds_count{{{labels}}} {}\n",
                metric.count
            ));
        }
        output
    }
}

/// Keeps the OTLP provider alive and flushes queued spans at orderly shutdown.
pub struct TracingGuard {
    provider: Option<SdkTracerProvider>,
}

impl Drop for TracingGuard {
    fn drop(&mut self) {
        if let Some(Err(error)) = self.provider.take().map(|provider| provider.shutdown()) {
            eprintln!("failed to flush OTLP spans during shutdown: {error}");
        }
    }
}

#[derive(Debug)]
/// Failure to install the configured tracing subscriber or OTLP exporter.
pub struct TracingInstallError(String);

impl fmt::Display for TracingInstallError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for TracingInstallError {}

impl Default for ObservabilityRuntime {
    fn default() -> Self {
        Self::new(ObservabilityConfig::Local)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct RequestSeries {
    method: &'static str,
    route: String,
    status_class: &'static str,
}

impl RequestSeries {
    fn labels(&self) -> String {
        format!(
            "method=\"{}\",route=\"{}\",status_class=\"{}\"",
            self.method,
            escape_label(&self.route),
            self.status_class
        )
    }
}

#[derive(Debug, Clone, Default)]
struct RequestMetric {
    count: u64,
    duration_sum_seconds: f64,
    buckets: [u64; DURATION_BUCKETS_SECONDS.len()],
}

impl RequestMetric {
    fn record(&mut self, duration: Duration) {
        let seconds = duration.as_secs_f64();
        self.count += 1;
        self.duration_sum_seconds += seconds;
        for (index, bound) in DURATION_BUCKETS_SECONDS.iter().enumerate() {
            if seconds <= *bound {
                self.buckets[index] += 1;
            }
        }
    }
}

fn bounded_method(method: &str) -> &'static str {
    match method {
        "GET" => "GET",
        "POST" => "POST",
        "PUT" => "PUT",
        "PATCH" => "PATCH",
        "DELETE" => "DELETE",
        "HEAD" => "HEAD",
        "OPTIONS" => "OPTIONS",
        _ => "OTHER",
    }
}

fn bounded_route(route: &str) -> String {
    if route.starts_with('/') && route.len() <= 160 {
        route.to_owned()
    } else {
        "unmatched".to_owned()
    }
}

const fn status_class(status: u16) -> &'static str {
    match status {
        200..=299 => "2xx",
        300..=399 => "3xx",
        400..=499 => "4xx",
        500..=599 => "5xx",
        _ => "other",
    }
}

fn escape_label(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}
