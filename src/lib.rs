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

use async_trait::async_trait;
use opentelemetry::{
    global, sdk::propagation::TraceContextPropagator, sdk::trace, sdk::Resource, trace::TraceId,
    KeyValue,
};
use opentelemetry_otlp::WithExportConfig;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter, Registry};

use std::borrow::Cow;

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
        let formatting_layer = BunyanFormattingLayer::new(self.app_name.clone(), std::io::stdout);
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
                    .install_batch(opentelemetry::runtime::Tokio)?;
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
                        .with(JsonStorageLayer)
                        .with(formatting_layer);
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
                        .with(JsonStorageLayer)
                        .with(formatting_layer)
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
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
