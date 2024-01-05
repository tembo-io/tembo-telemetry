//! `tembo-telemetry` is a crate that provides logging and telemetry exporters for Tembo.io applications.
//! It integrates with the OpenTelemetry ecosystem to provide detailed traces.
//!
//! # Features
//! - Configurable telemetry setup via `TelemetryConfig`.
//! - Integration with the OpenTelemetry and tracing ecosystems.
//! - Out-of-the-box support for OTLP exporters.
//! - Environment-specific logger configurations.
//!
//! # Usage
//! Refer to the `TelemetryConfig` and `TelemetryInit` traits for setting up and initializing telemetry.

use actix_web::{
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
    Error,
};
use async_trait::async_trait;
use opentelemetry::{global, trace::TraceId, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
    propagation::TraceContextPropagator, runtime::TokioCurrentThread, trace, Resource,
};
use tracing::Span;
use tracing_actix_web::{DefaultRootSpanBuilder, RootSpanBuilder, TracingLogger};
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    EnvFilter, Registry,
};

use std::{borrow::Cow, cell::RefCell};

/// Configuration for telemetry setup.
///
/// This struct provides fields to set up OpenTelemetry exporters, specify the application name,
/// environment, endpoint URL, and an optional tracer ID.
#[derive(Clone, Default, Debug)]
pub struct TelemetryConfig {
    /// Name of the application.
    pub app_name: String,
    /// Specifies the environment (e.g., "development" or "production").
    pub env: String,
    /// Optional URL for the OTLP exporter.
    pub endpoint_url: Option<String>,
    /// Optional tracer ID.
    pub tracer_id: Option<String>,
}

/// Trait to initialize telemetry based on the provided configuration.
#[async_trait]
pub trait TelemetryInit {
    /// Initializes telemetry based on the configuration.
    ///
    /// This method sets up the global tracer provider, OTLP exporter (if specified),
    /// and logger based on the environment.
    async fn init(&self) -> Result<(), Box<dyn std::error::Error>>;
}

impl TelemetryConfig {
    /// Retrieves the current trace ID.
    ///
    /// This method fetches the trace ID from the current span context.
    pub fn get_trace_id(&self) -> TraceId {
        use opentelemetry::trace::TraceContextExt as _; // opentelemetry::Context -> opentelemetry::trace::Span
        use tracing_opentelemetry::OpenTelemetrySpanExt as _; // tracing::Span to opentelemetry::Context

        tracing::Span::current()
            .context()
            .span()
            .span_context()
            .trace_id()
    }
}

/// Initializes telemetry based on the provided configuration.
///
/// This method will:
/// - Set the global text map propagator to `TraceContextPropagator`.
/// - Check for an OTLP endpoint and set up the OTLP exporter if present.
/// - Configure a logger based on the environment (`development` or other).
/// - Optionally, set a global tracer if `tracer_id` is provided.
#[async_trait]
impl TelemetryInit for TelemetryConfig {
    async fn init(&self) -> Result<(), Box<dyn std::error::Error>> {
        let env_filter =
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
        let sampler = trace::Sampler::AlwaysOn;
        let resource = Resource::new(vec![KeyValue::new("service.name", self.app_name.clone())]);
        let trace_config = trace::config()
            .with_sampler(sampler)
            .with_resource(resource);
        global::set_text_map_propagator(TraceContextPropagator::new());

        // Check if OPENTELEMERTY_OTLP_ENDPOINT is set, if not enable standard logger
        match &self.endpoint_url {
            Some(endpoint_url) => {
                let exporter = opentelemetry_otlp::new_exporter()
                    .tonic()
                    .with_endpoint(endpoint_url);
                let tracer = opentelemetry_otlp::new_pipeline()
                    .tracing()
                    .with_exporter(exporter)
                    .with_trace_config(trace_config)
                    .install_batch(TokioCurrentThread)?;
                let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
                if self.env == "development" {
                    let logger = fmt::layer().compact();
                    let subscriber = Registry::default()
                        .with(telemetry)
                        .with(logger)
                        .with(env_filter);
                    tracing::subscriber::set_global_default(subscriber)
                        .expect("setting default subscriber failed");
                } else {
                    let subscriber = Registry::default()
                        .with(telemetry)
                        .with(env_filter)
                        .with(fmt::layer().json().with_span_events(FmtSpan::NONE));
                    tracing::subscriber::set_global_default(subscriber)
                        .expect("setting default subscriber failed");
                };
            }
            None => {
                if self.env == "development" {
                    let logger = fmt::layer().compact();
                    let subscriber = Registry::default().with(logger).with(env_filter);
                    tracing::subscriber::set_global_default(subscriber)
                        .expect("setting default subscriber failed");
                } else {
                    let subscriber = Registry::default()
                        .with(fmt::layer().json().with_span_events(FmtSpan::NONE))
                        .with(env_filter);
                    tracing::subscriber::set_global_default(subscriber)
                        .expect("setting default subscriber failed");
                }
            }
        }
        if let Some(tracer_id) = &self.tracer_id {
            let name: Cow<'static, str> = tracer_id.to_string().into();
            global::tracer(name);
        }

        // Setup bridge between tracing crate and the log crate.  If someone
        // uses this crate, then if they use the log crate, they will get
        // the logs printed into the tracing session.
        tracing_log::LogTracer::init()?;
        Ok(())
    }
}

thread_local! {
    /// Thread-local storage for excluded routes.
    ///
    /// Contains a list of routes (endpoints) that should not be logged.
    static EXCLUDED_ROUTES: RefCell<Vec<String>> = RefCell::new(Vec::new());
}

/// Custom root span builder that allows for filtering out specific routes.
///
/// This builder will check if a request's path is in the list of excluded routes,
/// and if so, it won't log that request.
pub struct CustomFilterRootSpanBuilder;

impl CustomFilterRootSpanBuilder {
    /// Sets the routes to be excluded from logging.
    ///
    /// # Arguments
    ///
    /// * `routes` - A list of route paths to exclude.
    pub fn set_excluded_routes(routes: Vec<String>) {
        EXCLUDED_ROUTES.with(|excluded| {
            *excluded.borrow_mut() = routes;
        });
    }
}

impl RootSpanBuilder for CustomFilterRootSpanBuilder {
    fn on_request_start(request: &ServiceRequest) -> Span {
        let should_exclude = EXCLUDED_ROUTES
            .with(|excluded| excluded.borrow().contains(&request.path().to_string()));

        if should_exclude {
            Span::none()
        } else {
            tracing_actix_web::root_span!(level = tracing::Level::INFO, request)
        }
    }

    fn on_request_end<B: MessageBody>(span: Span, outcome: &Result<ServiceResponse<B>, Error>) {
        DefaultRootSpanBuilder::on_request_end(span, outcome);
    }
}

/// Builder for creating a custom logging middleware.
///
/// This builder provides methods to specify which routes to exclude from logging.
pub struct CustomLoggerBuilder {
    excluded_routes: Vec<String>,
}

impl CustomLoggerBuilder {
    /// Creates a new instance of `CustomLoggerBuilder` with no excluded routes.
    pub fn new() -> Self {
        Self {
            excluded_routes: Vec::new(),
        }
    }

    /// Specifies a route to be excluded from logging.
    ///
    /// # Arguments
    ///
    /// * `route` - The path of the route to exclude.
    pub fn exclude(mut self, route: &str) -> Self {
        self.excluded_routes.push(route.to_string());
        self
    }

    /// Builds and returns a custom logging middleware.
    ///
    /// This middleware will use `CustomFilterRootSpanBuilder` to filter out the specified routes.
    pub fn build(self) -> TracingLogger<CustomFilterRootSpanBuilder> {
        // Set the excluded routes for our custom builder
        CustomFilterRootSpanBuilder::set_excluded_routes(self.excluded_routes);

        // Return a TracingLogger with our custom builder
        TracingLogger::<CustomFilterRootSpanBuilder>::new()
    }
}

impl Default for CustomLoggerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function to obtain a `CustomLoggerBuilder`.
///
/// This can be used to start the builder chain for constructing the custom logger.
pub fn get_tracing_logger() -> CustomLoggerBuilder {
    CustomLoggerBuilder::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test::TestRequest;

    #[test]
    fn test_telemetry_config_defaults() {
        let config = TelemetryConfig::default();
        assert_eq!(config.app_name, "");
        assert_eq!(config.env, "");
        assert!(config.endpoint_url.is_none());
        assert!(config.tracer_id.is_none());
    }

    #[tokio::test]
    async fn test_init_with_defaults() {
        let config = TelemetryConfig::default();
        let result = config.init().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_excluded_route() {
        CustomFilterRootSpanBuilder::set_excluded_routes(vec!["/health/liveness".to_string()]);
        let req = TestRequest::get().uri("/health/liveness").to_srv_request();
        let span = CustomFilterRootSpanBuilder::on_request_start(&req);
        assert!(span.is_none());
    }

    #[tokio::test]
    async fn test_non_excluded_route() {
        fn mock_on_request_start(request: &ServiceRequest) -> bool {
            let should_exclude = EXCLUDED_ROUTES
                .with(|excluded| excluded.borrow().contains(&request.path().to_string()));
            !should_exclude
        }
        CustomFilterRootSpanBuilder::set_excluded_routes(vec!["/health/liveness".to_string()]);
        let req = TestRequest::get().uri("/some/other/route").to_srv_request();
        let should_log = mock_on_request_start(&req);
        assert!(should_log);
    }
}
