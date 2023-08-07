<!-- ![Tembo.io — Logging and Telemetry for Rust applications.](https://path.to/your/logo.png) -->

# Tembo Telemetry

Logging and Telemetry exporters for [Tembo.io](https://tembo.io) applications.

[![Crates.io: tembo-telemetry](https://img.shields.io/crates/v/tembo-telemetry.svg)](https://crates.io/crates/tembo-telemetry)
[![Documentation](https://docs.rs/tembo-telemetry/badge.svg)](https://docs.rs/tembo-telemetry)
[![LICENSE](https://img.shields.io/crates/l/tembo-telemetry)](./LICENSE)
[![GitHub Actions CI](https://github.com/tembo-io/tembo-telemetry/workflows/CI/badge.svg)](https://github.com/tembo-io/tembo-telemetry/actions?query=workflow%3ACI+branch%3Amain)
[![Tembo Slack](https://img.shields.io/badge/slack-@tembo/rust-brightgreen.svg?logo=slack)](https://tembocommunity.slack.com)

## Overview

[Tembo Telemetry](https://github.com/tembo-io/tembo-telemetry) is a Rust crate designed to easily integrate logging and telemetry capabilities into applications built with Rust. It provides a streamlined way to configure and send telemetry data, with support for the OTLP telemetry format.

## Quickstart

To get started with `tembo-telemetry`, add it as a dependency in your `Cargo.toml`:

```toml
[dependencies]
tembo-telemetry = "0.1.0"
```

Then, in your main application, set up and initialize the telemetry system:

```rust
use tembo_telemetry::{TelemetryConfig, TelemetryInit};

async fn main() {
    let telemetry_config = TelemetryConfig {
        app_name: "my_app".to_string(),
        env: "production".to_string(),
        endpoint_url: Some("http://my-telemetry-endpoint".to_string()),
        tracer_id: Some("my_tracer_id".to_string()),
    };

    let _ = telemetry_config.init().await;
}
```
