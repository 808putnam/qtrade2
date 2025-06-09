use anyhow::Result;
use clap::{Parser, ValueEnum};
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

#[derive(Parser)]
#[command(version)]
struct Cli {
    #[arg(short, long, value_enum)]
    blockchain: crate::Blockchain,

    #[arg(short, long, value_enum)]
    router: crate::Router,

    #[arg(long = "config", value_name = "TOML_CONFIG_FILE",
          help = "Path to TOML configuration file. If not specified, will look for qtrade.toml in the current directory")]
    config_file: Option<PathBuf>,

    #[arg(short, long = "vixen", value_name = "VIXEN_CONFIG_FILE")]
    vixen_config: PathBuf,

    // API key override flags (all optional)
    #[arg(long, value_name = "BLOXROUTE_API_KEY")]
    bloxroute_api_key: Option<String>,

    #[arg(long, value_name = "HELIUS_API_KEY")]
    helius_api_key: Option<String>,

    #[arg(long, value_name = "NEXTBLOCK_API_KEY")]
    nextblock_api_key: Option<String>,

    #[arg(long, value_name = "QUICKNODE_API_KEY")]
    quicknode_api_key: Option<String>,

    #[arg(long, value_name = "TEMPORAL_API_KEY")]
    temporal_api_key: Option<String>,

    // Active RPC providers flag (comma-separated list of providers to use)
    #[arg(long = "active-rpcs", value_name = "RPC_PROVIDERS",
          help = "Comma-separated list of RPC providers to use. Available options: bloxroute, helius, jito, nextblock, quicknode, solana, temporal, triton",
          value_delimiter = ',')]
    active_rpcs: Option<Vec<String>>,

    // Active DEX platforms flag (comma-separated list of DEXes to use)
    #[arg(long = "active-dexes", value_name = "DEX_PLATFORMS",
          help = "Comma-separated list of DEX platforms to use. Available options: orca, raydium, raydium-cpmm, raydium-clmm",
          value_delimiter = ',')]
    active_dexes: Option<Vec<String>>,

    // Single wallet mode for testing and debugging
    #[arg(long, help = "Use a single wallet instead of the multi-tiered wallet system")]
    single_wallet: bool,

    #[arg(long, value_name = "PRIVATE_KEY", help = "Private key for the single wallet mode")]
    single_wallet_private_key: Option<String>,

    // Transaction simulation flag
    #[arg(long, help = "Simulate transactions instead of submitting them to the network")]
    simulate: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Blockchain {
    #[clap(rename_all = "lower")]
    Solana,
    #[clap(rename_all = "lower")]
    Sui,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Router {
    #[clap(rename_all = "lower")]
    Cvxpy,
    #[clap(rename_all = "lower")]
    CFMMRouter,
    #[clap(rename_all = "lower")]
    OpenQAOA,
}

struct ClientConfig {
    flags: qtrade_runtime::settings::Flags,
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
        .with_service_name("qtrade-client") // Add service.name resource attribute.
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
            qtrade_runtime::run_qtrade(
                cfg.flags,
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
    let cli = Cli::parse();

    let blockchain = match cli.blockchain {
        crate::Blockchain::Solana => {
            qtrade_runtime::Blockchain::Solana
        }
        crate::Blockchain::Sui => {
            qtrade_runtime::Blockchain::Sui
        }
    };

    let router = match cli.router {
        crate::Router::CFMMRouter => {
            qtrade_runtime::Router::CFMMRouter
        }
        crate::Router::Cvxpy => {
            qtrade_runtime::Router::Cvxpy
        }
        crate::Router::OpenQAOA => {
            qtrade_runtime::Router::OpenQAOA
        }
    };

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

    // Create the Flags struct from CLI arguments
    let flags = qtrade_runtime::settings::Flags {
        config_file_path: cli.config_file.map(|p| p.to_string_lossy().into_owned()),
        vixon_config_path: Some(cli.vixen_config.to_string_lossy().into_owned()),
        bloxroute_api_key: cli.bloxroute_api_key,
        helius_api_key: cli.helius_api_key,
        nextblock_api_key: cli.nextblock_api_key,
        quicknode_api_key: cli.quicknode_api_key,
        temporal_api_key: cli.temporal_api_key,
        active_rpcs: cli.active_rpcs,
        active_dexes: cli.active_dexes,
        single_wallet: cli.single_wallet,
        single_wallet_private_key: cli.single_wallet_private_key,
        blockchain: Some(blockchain.clone()),
        router: Some(router.clone()),
        simulate: cli.simulate,
    };

    Ok(ClientConfig {
        flags,
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
