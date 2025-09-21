use crate::CliContext;
use anyhow::{Context, Result};
use lact_schema::args::cli::PowerLimitCmd;

pub async fn list_gpus(ctx: CliContext<'_>) -> Result<()> {
    let entries = ctx.client.list_devices().await?;
    for (i, entry) in entries.into_iter().enumerate() {
        let id = entry.id;
        let device_type = entry.device_type;

        if let Some(name) = entry.name {
            println!("{i}: {id} ({name}) [{device_type}]");
        } else {
            println!("{i}: {id} [{device_type}]");
        }
    }
    Ok(())
}

pub async fn info(ctx: CliContext<'_>) -> Result<()> {
    let id = ctx.current_gpu_id().await?;

    let gpu_line = format!("GPU {id}:");
    println!("{gpu_line}");
    println!("{}", "=".repeat(gpu_line.len()));

    let info = ctx.client.get_device_info(&id).await?;
    let stats = ctx.client.get_device_stats(&id).await?;

    let elements = info.info_elements(Some(&stats));
    for (name, value) in elements {
        if let Some(value) = value {
            println!("{name}: {value}");
        }
    }

    Ok(())
}

pub async fn snapshot(ctx: CliContext<'_>) -> Result<()> {
    let path = ctx.client.generate_debug_snapshot().await?;
    println!("Generated debug snapshot in {path}");
    Ok(())
}

pub async fn power_limit(ctx: CliContext<'_>, cmd: Option<PowerLimitCmd>) -> Result<()> {
    let id = ctx.current_gpu_id().await?;
    match cmd {
        Some(PowerLimitCmd::Get) | None => {
            let stats = ctx.client.get_device_stats(&id).await?;
            let cap = stats
                .power
                .cap_current
                .context("No cap reported by the GPU")?;

            print!("Current power limit: {cap}W");

            if let (Some(min), Some(max)) = (stats.power.cap_min, stats.power.cap_max) {
                print!(" (Configurable Range: {min}W to {max}W)");
            }
            println!();
        }
        Some(PowerLimitCmd::Set { limit }) => {
            ctx.edit_gpu_config(&id, |config| {
                config.power_cap = Some(limit.into());
            })
            .await?;
            println!("Updated power limit to {limit}W");
        }
    }
    Ok(())
}
