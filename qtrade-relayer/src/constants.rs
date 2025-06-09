use once_cell::sync::Lazy;
use opentelemetry::{InstrumentationScope, metrics::Meter, global};

// Our one global named tracer we will use throughout the relayer
pub const QTRADE_RELAYER_TRACER_NAME: &str = "qtrade_relayer";
pub const QTRADE_RELAYER: &str = "qtrade_relayer";

pub static QTRADE_RELAYER_SCOPE: Lazy<InstrumentationScope> = Lazy::new(|| {
    InstrumentationScope::builder(QTRADE_RELAYER_TRACER_NAME)
        .with_version(env!("CARGO_PKG_VERSION"))
        .with_schema_url("https://opentelemetry.io/schemas/1.17.0")
        .build()
});

pub static QTRADE_RELAYER_METER: Lazy<Meter> = Lazy::new(|| {
    global::meter(QTRADE_RELAYER)
});
