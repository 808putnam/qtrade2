use anyhow::Result;
use opentelemetry::global;
use opentelemetry::KeyValue;
use opentelemetry::trace::Tracer;
use std::process::Command;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, error};

// For help in naming spans
use crate::checks::QTRADE_CUSTOM_CHECKS_TRACER_NAME;
use crate::checks::QTRADE_CUSTOM_CHECKS_METER;

const CHECK_SYSTEMD_HEALTHCHECKS: &str = "check_systemd_healthchecks";
const CHECK_INTERVAL: Duration = Duration::from_secs(60);

// ai: You should use qtrade-custom-checks.service in the SERVICES array.
// This is because the systemctl commands (is-active and is-failed) expect
// the full service name, including the .service suffix, to correctly identify
// and check the status of the service.
const SERVICES: &[&str] = &["qtrade-custom-checks.service"];

pub async fn run_check_systemd_healthchecks() -> Result<()> {
    let tracer = global::tracer(QTRADE_CUSTOM_CHECKS_TRACER_NAME);

    loop {
        let span_name = format!("{}::run_check_systemd_healthchecks", CHECK_SYSTEMD_HEALTHCHECKS);
        let result: Result<(), anyhow::Error> = tracer.in_span(span_name, |_cx| async move {
            info!("Checking systemd healthchecks");

            let check_systemd_healthchecks_running_instrument = QTRADE_CUSTOM_CHECKS_METER
                .i64_gauge("check_systemd_healthchecks_running")
                .with_description("Systemd running healthchecks")
                .build();

            let check_systemd_healthchecks_healthy_instrument = QTRADE_CUSTOM_CHECKS_METER
                .i64_gauge("check_systemd_healthchecks_healthy")
                .with_description("Systemd healthy healthchecks")
                .build();

            for &service in SERVICES {
                let service_short_name = service.split('.').next().unwrap_or(service);
                let is_active = is_service_active(service);
                let is_healthy = is_service_healthy(service);

                info!(service_name = service_short_name, running = is_active, healthy = is_healthy, "Service status checked");
                check_systemd_healthchecks_running_instrument.record(is_active as i64, &[KeyValue::new("service", service_short_name)]);
                check_systemd_healthchecks_healthy_instrument.record(is_healthy as i64, &[KeyValue::new("service", service_short_name)]);
            }

            Ok(())
        }).await;

        if let Err(e) = result {
            error!("Error running check: {:?}", e);
        }

        // Wait for specified duration before running the check again
        sleep(CHECK_INTERVAL).await;
    }
}

fn is_service_active(service_name: &str) -> bool {
    match Command::new("systemctl").arg("is-active").arg(service_name).output() {
        Ok(output) => String::from_utf8_lossy(&output.stdout).trim() == "active",
        Err(_) => false,
    }
}

fn is_service_healthy(service_name: &str) -> bool {
    match Command::new("systemctl").arg("is-failed").arg(service_name).output() {
        Ok(output) => String::from_utf8_lossy(&output.stdout).trim() != "failed",
        Err(_) => false,
    }
}
