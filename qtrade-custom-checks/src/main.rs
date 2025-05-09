use anyhow::Result;
use once_cell::sync::Lazy;
use opentelemetry::{
    global,
    trace::{TraceContextExt, Tracer},
    KeyValue,
    InstrumentationScope,
};
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::{LogExporter, MetricExporter, SpanExporter};
use opentelemetry_sdk::{
    logs::SdkLoggerProvider,
    metrics::SdkMeterProvider,
    trace::SdkTracerProvider,
    Resource,
};
use std::{env, path::PathBuf};
use tokio::{select, signal};
use tokio_util::sync::CancellationToken;
use tracing::info;
use tracing_subscriber::{EnvFilter, prelude::*};

pub mod checks;

struct ClientConfig {
    logger_provider: SdkLoggerProvider,
    meter_provider: SdkMeterProvider,
    tracer_provider: SdkTracerProvider,
}

// An immutable representation of the entity producing telemetry as attributes. Utilizes Arc for efficient sharing and cloning.
// Creates a ResourceBuilder that allows you to configure multiple aspects of the Resource.
// This ResourceBuilder will include the following ResourceDetectors:
// - SdkProvidedResourceDetector 
//   - https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/resource/struct.SdkProvidedResourceDetector.html
//   - https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/resource/sdk.md#sdk-provided-resource-attributes
//   - https://github.com/open-telemetry/semantic-conventions/blob/main/docs/resource/README.md#semantic-attributes-with-sdk-provided-default-value
// - TelemetryResourceDetector 
//   - https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/resource/struct.TelemetryResourceDetector.html
// - EnvResourceDetector 
//   - https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/resource/struct.EnvResourceDetector.html
//   - https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/resource/sdk.md#specifying-resource-information-via-an-environment-variable
//
static RESOURCE: Lazy<Resource> = Lazy::new(|| {
    Resource::builder()
        .with_service_name("qtrade-custom-checks") // Add service.name resource attribute.
        .build()
});

#[tokio::main]
async fn main() -> Result<()> {
    let cfg = init_config().map_err(|e| {
        eprintln!("Init config failed with error: {:?}", e);
        e
    })?;

    let token = CancellationToken::new();
    let cloned_token = token.clone();
    let shutdown_signal = async {
        signal::ctrl_c().await.map_err(|e| {
            anyhow::anyhow!("Failed to install Ctrl+C handler: {:?}", e)
        })
    };

    select! {
        _ = shutdown_signal => {
            info!("Received Ctrl-C, shutting down...");
            token.cancel();
        }
        result = async {
            crate::checks::run_custom_checks(
                cloned_token).await
        } => {
            result?;
        }
    }

    cfg.tracer_provider.shutdown()?;
    cfg.meter_provider.shutdown()?;
    cfg.logger_provider.shutdown()?;

    Ok(())
}

fn init_config() -> Result<ClientConfig> {

    let logger_provider = init_logs();

    // Create a new OpenTelemetryTracingBridge using the above LoggerProvider.
    let otel_layer = OpenTelemetryTracingBridge::new(&logger_provider);

    // For the OpenTelemetry layer, add a tracing filter to filter events from
    // OpenTelemetry and its dependent crates (opentelemetry-otlp uses crates
    // like reqwest/tonic etc.) from being sent back to OTel itself, thus
    // preventing infinite telemetry generation. The filter levels are set as
    // follows:
    // - Allow `info` level and above by default.
    // - Restrict `opentelemetry`, `hyper`, `tonic`, and `reqwest` completely.
    // Note: This will also drop events from crates like `tonic` etc. even when
    // they are used outside the OTLP Exporter. For more details, see:
    // https://github.com/open-telemetry/opentelemetry-rust/issues/761
    let filter_otel = EnvFilter::new("info")
        .add_directive("hyper=off".parse().unwrap())
        .add_directive("opentelemetry=off".parse().unwrap())
        .add_directive("tonic=off".parse().unwrap())
        .add_directive("h2=off".parse().unwrap())
        .add_directive("reqwest=off".parse().unwrap());
    let otel_layer = otel_layer.with_filter(filter_otel);

    // Read maximum log levels from the environment, using info if it's missing or
    // we can't parse it.
    let log_level = env::var("LOG_LEVEL")
        .unwrap_or_else(|_| "info".to_string());
    let otel_log_level = env::var("OTEL_LOG_LEVEL")
        .unwrap_or_else(|_| "info".to_string());
    let otel_log_directive = format!("opentelemetry={}", otel_log_level);
    
    // Create a new tracing::Fmt layer to print the logs to stdout. It has a
    // default filter of `info` level and above, and `debug` and above for logs
    // from OpenTelemetry crates.
    // We currently read from env. vars with defaults of info for both for this.
    let filter_fmt = EnvFilter::new(log_level).add_directive(otel_log_directive.parse().unwrap());
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_thread_names(true)
        .with_filter(filter_fmt);

    // Initialize the tracing subscriber with the OpenTelemetry layer and the
    // Fmt layer.
    tracing_subscriber::registry()
        .with(otel_layer)
        .with(fmt_layer)
        .init();

    // At this point Logs (OTel Logs and Fmt Logs) are initialized, which will
    // allow internal-logs from Tracing/Metrics initializer to be captured.

    // Creator and registry of named SdkTracer instances. 
    // https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/trace/struct.SdkTracerProvider.html
    let tracer_provider = init_traces();
    // Set the global tracer provider using a clone of the tracer_provider.
    // Setting global tracer provider is required if other parts of the application
    // uses global::tracer() or global::tracer_with_version() to get a tracer.
    // Cloning simply creates a new reference to the same tracer provider. It is
    // important to hold on to the tracer_provider here, so as to invoke
    // shutdown on it when application ends.
    global::set_tracer_provider(tracer_provider.clone());

    let meter_provider = init_metrics();
    // Set the global meter provider using a clone of the meter_provider.
    // Setting global meter provider is required if other parts of the application
    // uses global::meter() or global::meter_with_version() to get a meter.
    // Cloning simply creates a new reference to the same meter provider. It is
    // important to hold on to the meter_provider here, so as to invoke
    // shutdown on it when application ends.
    global::set_meter_provider(meter_provider.clone());

    Ok(ClientConfig {
        logger_provider,
        meter_provider,
        tracer_provider
    })
}

fn otel_reference() {
    // Information about a library or crate providing instrumentation.
    // https://docs.rs/opentelemetry/latest/opentelemetry/global/index.html#usage-in-libraries
    // https://docs.rs/opentelemetry/latest/opentelemetry/struct.InstrumentationScope.html
    // https://github.com/open-telemetry/opentelemetry-specification/blob/v1.9.0/specification/overview.md#instrumentation-libraries
    let common_scope_attributes = vec![KeyValue::new("scope-key", "scope-value")];
    let scope = InstrumentationScope::builder("basic")
        .with_version("1.0")
        .with_attributes(common_scope_attributes)
        .build();

    // https://docs.rs/opentelemetry/latest/opentelemetry/global/index.html
    // https://docs.rs/opentelemetry/latest/opentelemetry/global/index.html#usage-in-libraries
    // https://docs.rs/opentelemetry/latest/opentelemetry/global/index.html#global-metrics-api
    // https://docs.rs/opentelemetry/latest/opentelemetry/global/index.html#usage-in-applications-and-libraries
    let tracer = global::tracer_with_scope(scope.clone());
    let meter = global::meter_with_scope(scope);

    let counter = meter
        .u64_counter("test_counter")
        .with_description("a simple counter for demo purposes.")
        .with_unit("my_unit")
        .build();
    for _ in 0..10 {
        counter.add(1, &[KeyValue::new("test_key", "test_value")]);
    }

    tracer.in_span("Main operation", |cx| {
        let span = cx.span();
        span.add_event(
            "Nice operation!".to_string(),
            vec![KeyValue::new("bogons", 100)],
        );
        span.set_attribute(KeyValue::new("another.key", "yes"));

        info!(name: "my-event-inside-span", target: "my-target", "hello from {}. My price is {}. I am also inside a Span!", "banana", 2.99);

        tracer.in_span("Sub operation...", |cx| {
            let span = cx.span();
            span.set_attribute(KeyValue::new("another.key", "yes"));
            span.add_event("Sub span event", vec![]);
        });
    });

    info!(name: "my-event", target: "my-target", "hello from {}. My price is {}", "apple", 1.99);
}

fn init_traces() -> SdkTracerProvider {
    // OTLP exporter that sends tracing information
    // https://docs.rs/opentelemetry-otlp/latest/opentelemetry_otlp/struct.SpanExporter.html
    // https://docs.rs/opentelemetry-otlp/latest/opentelemetry_otlp/struct.TonicExporterBuilder.html
    let exporter = SpanExporter::builder()
        .with_tonic()
        .build()
        .expect("Failed to create span exporter");
    
    // Creator and registry of named SdkTracer instances. 
    // https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/trace/struct.SdkTracerProvider.html
    SdkTracerProvider::builder()
        .with_resource(RESOURCE.clone())
        .with_batch_exporter(exporter)
        .build()
}

fn init_metrics() -> SdkMeterProvider {
    let exporter = MetricExporter::builder()
        .with_tonic()
        .build()
        .expect("Failed to create metric exporter");

    SdkMeterProvider::builder()
        .with_periodic_exporter(exporter)// Defaults to 60 seconds
        .with_resource(RESOURCE.clone())
        .build()
}

fn init_logs() -> SdkLoggerProvider {
    let exporter = LogExporter::builder()
        .with_tonic()
        .build()
        .expect("Failed to create log exporter");

    SdkLoggerProvider::builder()
        .with_resource(RESOURCE.clone())
        .with_batch_exporter(exporter)
        .build()
}
