mod schema;

use crate::{
    config,
    server::{handler::Handler, metrics::schema::NumberValue},
};
use chrono::Local;
use indexmap::IndexMap;
use lact_schema::DeviceStats;
use schema::{
    Attribute, Gauge, GaugeDataPoint, Metric, MetricsPayload, Resource, ResourceMetric, Scope,
    ScopeMetric, Value,
};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, error, info};

pub fn setup(handler: Handler, config: config::Metrics) {
    info!(
        "exporting metrics to {} every {} seconds",
        config.collector_address, config.interval
    );

    let interval = Duration::from_secs(config.interval);

    tokio::task::spawn_local(async move {
        let client = ureq::config::Config::builder()
            .http_status_as_error(false)
            .build()
            .new_agent();

        loop {
            sleep(interval).await;

            match get_stats(&handler).await {
                Ok(devices) => {
                    debug!("collecting metrics for {} devices", devices.len());
                    let mut metrics = Vec::with_capacity(10);
                    let timestamp = Local::now()
                        .timestamp_nanos_opt()
                        .expect("Invalid timestamp")
                        .to_string();

                    for (gpu_id, (gpu_name, stats)) in &devices {
                        collect_metrics(gpu_id, gpu_name, stats, &mut metrics, &timestamp);
                    }

                    let metric_count = metrics.len();

                    let request = MetricsPayload {
                        resource_metrics: vec![ResourceMetric {
                            resource: Resource {
                                attributes: vec![Attribute {
                                    key: "service.name",
                                    value: Value::String("LACT"),
                                }],
                            },
                            scope_metrics: vec![ScopeMetric {
                                scope: Scope {
                                    name: "LACT".to_owned(),
                                    version: env!("CARGO_PKG_VERSION").to_owned(),
                                    attributes: vec![],
                                },
                                metrics,
                            }],
                        }],
                    };

                    let url = config.collector_address.clone();
                    let client = client.clone();
                    let payload =
                        serde_json::to_string(&request).expect("Could not serialize request body");

                    let response = tokio::task::spawn_blocking(move || {
                        client
                            .post(url)
                            .content_type("application/json")
                            .send(payload)
                    })
                    .await
                    .unwrap();

                    match response {
                        Ok(response) => {
                            if response.status().is_success() {
                                debug!("{metric_count} metrics were uploaded");
                            } else {
                                let status = response.status();
                                let body = response
                                    .into_body()
                                    .read_to_string()
                                    .unwrap_or_else(|_| "<Invalid string>".to_owned());
                                debug!("could not submit metrics, status code {status}: {body}");
                            }
                        }
                        Err(err) => error!("could not submit metrics: {err}"),
                    }
                }
                Err(err) => error!("could not fetch metrics: {err:#}"),
            }
        }
    });
}

#[allow(
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::too_many_lines
)]
fn collect_metrics<'a>(
    gpu_id: &'a str,
    gpu_name: &'a str,
    stats: &'a DeviceStats,
    metrics: &mut Vec<Metric<'a>>,
    timestamp: &'a str,
) {
    let base_attrs = vec![
        Attribute {
            key: "gpu_id",
            value: Value::String(gpu_id),
        },
        Attribute {
            key: "gpu_name",
            value: Value::String(gpu_name),
        },
    ];

    if let Some(usage) = stats.busy_percent {
        metrics.push(make_metric(
            "lact_gpu_usage",
            i64::from(usage).into(),
            "%",
            "Current GPU usage",
            timestamp,
            base_attrs.clone(),
        ));
    }

    if let Some(power) = stats.power.current.or(stats.power.average) {
        metrics.push(make_metric(
            "lact_gpu_power_usage",
            power.into(),
            "W",
            "Current power usage",
            timestamp,
            base_attrs.clone(),
        ));
    }

    if let Some(power_cap) = stats.power.cap_current {
        metrics.push(make_metric(
            "lact_gpu_power_usage_cap",
            power_cap.into(),
            "W",
            "Maximum configured power usage",
            timestamp,
            base_attrs.clone(),
        ));
    }

    for (key, temp) in &stats.temps {
        let mut temp_attrs = base_attrs.clone();
        temp_attrs.push(Attribute {
            key: "sensor",
            value: Value::String(key),
        });

        if let Some(current) = temp.value.current {
            metrics.push(make_metric(
                "lact_gpu_temperature",
                f64::from(current).into(),
                "Cel",
                "Current temperature",
                timestamp,
                temp_attrs.clone(),
            ));
        }

        if let Some(crit) = temp.value.crit {
            metrics.push(make_metric(
                "lact_gpu_temperature_crit",
                f64::from(crit).into(),
                "Cel",
                "Critical temperature",
                timestamp,
                temp_attrs.clone(),
            ));
        }
    }

    if let Some(gpu_clock) = stats.clockspeed.gpu_clockspeed {
        let mut attrs = base_attrs.clone();
        attrs.push(Attribute {
            key: "clockspeed_type",
            value: Value::String("gpu_current"),
        });

        metrics.push(make_metric(
            "lact_gpu_clockspeed",
            (gpu_clock as i64 * 1000 * 1000).into(),
            "Hz",
            "Current clockspeed",
            timestamp,
            attrs,
        ));
    }

    if let Some(gpu_target_clock) = stats.clockspeed.target_gpu_clockspeed {
        let mut attrs = base_attrs.clone();
        attrs.push(Attribute {
            key: "clockspeed_type",
            value: Value::String("gpu_target"),
        });

        metrics.push(make_metric(
            "lact_gpu_clockspeed",
            (gpu_target_clock as i64 * 1000 * 1000).into(),
            "Hz",
            "Current clockspeed",
            timestamp,
            attrs,
        ));
    }

    if let Some(vram_clock) = stats.clockspeed.vram_clockspeed {
        let mut attrs = base_attrs.clone();
        attrs.push(Attribute {
            key: "clockspeed_type",
            value: Value::String("memory"),
        });

        metrics.push(make_metric(
            "lact_gpu_clockspeed",
            (vram_clock as i64 * 1000 * 1000).into(),
            "Hz",
            "Current clockspeed",
            timestamp,
            attrs,
        ));
    }

    if let Some(gpu_voltage) = stats.voltage.gpu {
        metrics.push(make_metric(
            "lact_gpu_voltage",
            (gpu_voltage as f64 * 1000.0).into(),
            "V",
            "Current voltage",
            timestamp,
            base_attrs.clone(),
        ));
    }

    if let Some(fan_rpm) = stats.fan.speed_current {
        metrics.push(make_metric(
            "lact_gpu_fan_speed",
            i64::from(fan_rpm).into(),
            "RPM",
            "Current fan speed (RPM)",
            timestamp,
            base_attrs.clone(),
        ));
    }

    if let Some(fan_rpm_max) = stats.fan.speed_max {
        metrics.push(make_metric(
            "lact_gpu_fan_speed_max",
            i64::from(fan_rpm_max).into(),
            "RPM",
            "Maximum fan speed (RPM)",
            timestamp,
            base_attrs.clone(),
        ));
    }

    if let Some(fan_pwm) = stats.fan.pwm_current {
        let percent = f64::from(fan_pwm) / f64::from(u8::MAX) * 100.0;

        metrics.push(make_metric(
            "lact_gpu_fan_percent",
            percent.into(),
            "%",
            "Current fan speed (%)",
            timestamp,
            base_attrs.clone(),
        ));
    }

    if let Some(vram_used) = stats.vram.used {
        metrics.push(make_metric(
            "lact_gpu_vram_used",
            (vram_used as i64).into(),
            "By",
            "Current VRAM usage",
            timestamp,
            base_attrs.clone(),
        ));
    }

    if let Some(vram_total) = stats.vram.total {
        metrics.push(make_metric(
            "lact_gpu_vram_total",
            (vram_total as i64).into(),
            "By",
            "Total VRAM available",
            timestamp,
            base_attrs.clone(),
        ));
    }
}

fn make_metric<'a>(
    name: &'static str,
    value: NumberValue,
    unit: &'static str,
    description: &'static str,
    timestamp: &'a str,
    attrs: Vec<Attribute<'a>>,
) -> Metric<'a> {
    Metric {
        name,
        unit,
        description,
        gauge: Gauge {
            data_points: vec![GaugeDataPoint {
                value,
                time_unix_nano: timestamp,
                attributes: attrs,
            }],
        },
    }
}

async fn get_stats(handler: &Handler) -> anyhow::Result<IndexMap<String, (String, DeviceStats)>> {
    let mut devices = IndexMap::new();

    let device_list = handler.list_devices().await;

    for device in device_list {
        let stats = handler.get_gpu_stats(&device.id).await?;

        devices.insert(device.id, (device.name.unwrap_or_default(), stats));
    }

    Ok(devices)
}
