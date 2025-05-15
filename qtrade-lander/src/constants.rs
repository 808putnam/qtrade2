use once_cell::sync::Lazy;
use opentelemetry::{InstrumentationScope, metrics::Meter, global};

// Our one global named tracer we will use throughout the lander
pub const QTRADE_LANDER_TRACER_NAME: &str = "qtrade_lander";
pub const QTRADE_LANDER: &str = "qtrade_lander";

pub static QTRADE_LANDER_SCOPE: Lazy<InstrumentationScope> = Lazy::new(|| {
    InstrumentationScope::builder(QTRADE_LANDER_TRACER_NAME)
        .with_version(env!("CARGO_PKG_VERSION"))
        .with_schema_url("https://opentelemetry.io/schemas/1.17.0")
        .build()
});

pub static QTRADE_LANDER_METER: Lazy<Meter> = Lazy::new(|| {
    global::meter(QTRADE_LANDER)
});
