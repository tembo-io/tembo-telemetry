# Changelog

## vNext

## v0.3.0

### Added

- Added `tracing-log` crate to provide an adapter for connecting unstructured 
log records from the `log` crate into the `tracing` ecosystem. [#4](https://github.com/tembo-io/tembo-telemetry/pull/4)

### Changed

- Bump MSRV to 1.65 [#4](https://github.com/tembo-io/tembo-telemetry/pull/4)
- Updated `opentelemetry` crates to latest versions, which spilts the API and SDK
versions of the `opentelemetry` crate. []#4](https://github.com/tembo-io/tembo-telemetry/pull/4)
  - `opentelemetry` -> `0.21`
  - `opentelemetry-otlp` -> `0.14`
  - `opentelemetry_sdk` -> `0.21`
- Updated `tracing-opentelemetry` to `0.22`
- Updated `tracing-actix-web` to use feature `opentelemetry_0_21`
- Updated `actix-web` to `4.4`

## v0.2.0
