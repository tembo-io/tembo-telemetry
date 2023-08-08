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
tembo-telemetry = "*"
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

## Environment Configuration

The `tembo-telemetry` crate uses the `ENV` environment variable to determine the logging format suitable for different environments. This allows you to have tailored logging experiences for different deployment scenarios (e.g., development vs. production).

### Setting the `ENV` variable

To set the logging environment, you can set the `ENV` variable before running your application:

```bash
export ENV=production
```

### Logging Structure

- **Development (`ENV=development`)**: 
  In the development environment, logs are formatted for readability. They are concise and intended for local debugging purposes.

```bash
2023-08-08T01:33:27.046003Z  INFO actix_server::builder: starting 14 workers
2023-08-08T01:33:27.046046Z  INFO trace: Starting HTTP server at https://0.0.0.0:3001/
2023-08-08T01:33:27.046067Z  INFO actix_server::server: Actix runtime found; starting in Actix runtime
```
  
- **Production (`ENV=production` or any other value)**:
  In the production environment or any setting other than development, logs are structured in the JSON format. This structure is optimized for machine parsing and is suitable for log aggregation systems, monitoring, and alerting tools.

```bash
{"v":0,"name":"app","msg":"starting 14 workers","level":30,"hostname":"gwaihir","pid":300368,"time":"2023-08-08T01:34:19.145972575Z","target":"actix_server::builder","line":200,"file":"/home/nhudson/.cargo/registry/src/index.crates.io-6f17d22bba15001f/actix-server-2.2.0/src/builder.rs"}
{"v":0,"name":"app","msg":"Starting HTTP server at https://0.0.0.0:3001/","level":30,"hostname":"gwaihir","pid":300368,"time":"2023-08-08T01:34:19.146026447Z","target":"app","line":66,"file":"src/main.rs"}
{"v":0,"name":"app","msg":"Actix runtime found; starting in Actix runtime","level":30,"hostname":"gwaihir","pid":300368,"time":"2023-08-08T01:34:19.146055049Z","target":"actix_server::server","line":196,"file":"/home/nhudson/.cargo/registry/src/index.crates.io-6f17d22bba15001f/actix-server-2.2.0/src/server.rs"}
```

By default, if the `ENV` variable is not set, the logging will be in the non-development format.

To get the best logging experience tailored for your environment, always ensure to set the `ENV` variable appropriately before running your application.
