use std::process::ExitCode;
use std::sync::Arc;

use anyhow::{Context as _, Result as AnyResult};
use futures::prelude::*;
use smol::channel::Receiver as ChannelRx;

mod config;
mod controller;
mod output;

#[cfg(feature = "osc")]
mod osc;

#[cfg(feature = "vmc")]
mod vmc;

fn main() -> ExitCode {
    init_logger().expect("Failed to initialize logging");

    match run_sync() {
        Ok(()) => {
            log::debug!("Clean exit.");
            ExitCode::SUCCESS
        }

        Err(e) => {
            log::error!("An error has occurred: {:#}", e);
            log::logger().flush();

            eprintln!("Press any key to exit.");
            let _ = console::Term::stdout().read_key();

            ExitCode::FAILURE
        }
    }
}

fn run_sync() -> AnyResult<()> {
    smol::block_on(run_async())
}

async fn run_async() -> AnyResult<()> {
    let config = config::AppConfig::read_from("remote-wheel-sender.toml").await?;

    let (_cancel_tx, cancel_rx) = smol::channel::unbounded();
    let (value_tx, value_rx) = async_broadcast::broadcast(16);

    let exec = Arc::new(smol::Executor::new());
    let mut tasks = Vec::new();

    let _cancel_task = exec.spawn(run_cancel(cancel_rx.clone()));

    let controller_task = exec.spawn(controller::run(
        exec.clone(),
        config.mappings.clone(),
        value_tx.clone(),
        cancel_rx.clone(),
    ));
    tasks.push(controller_task);

    #[cfg(feature = "osc")]
    if config.osc.enabled() {
        let osc_task = exec.spawn(osc::run(
            exec.clone(),
            config.osc,
            config.mappings.clone(),
            cancel_rx.clone(),
            value_tx.clone(),
            value_rx.clone(),
        ));
        tasks.push(osc_task);
    }

    #[cfg(feature = "vmc")]
    if config.vmc.enabled() {
        let vmc_task = exec.spawn(vmc::run(
            config.vmc,
            config.mappings.clone(),
            value_rx.clone(),
        ));
        tasks.push(vmc_task);
    }

    drop(value_rx);
    drop(value_tx);

    exec.run(async move {
        let mut result = Ok(());

        while !tasks.is_empty() {
            let (task_result, _, rest) = futures::future::select_all(tasks).await;

            if let Err(ref e) = task_result {
                log::error!("Task failed: {e}");
                cancel_rx.close();
            }

            result = result.and(task_result);
            tasks = rest;
        }

        result
    })
    .await
}

fn init_logger() -> AnyResult<()> {
    let term_config = simplelog::ConfigBuilder::new()
        .set_time_offset_to_local()
        .unwrap_or_else(|e| e)
        .build();

    let term_logger = simplelog::TermLogger::new(
        log::LevelFilter::Info,
        term_config,
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Auto,
    );

    let file = std::fs::File::options()
        .create(true)
        .truncate(true)
        .write(true)
        .open("remote-wheel-sender.log")
        .context("Failed to open log file")?;

    let file_config = simplelog::ConfigBuilder::new()
        .set_time_format_custom(simplelog::format_description!(
            "[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]"
        ))
        .set_time_offset_to_local()
        .unwrap_or_else(|e| e)
        .build();

    let file_logger = simplelog::WriteLogger::new(log::LevelFilter::Trace, file_config, file);

    simplelog::CombinedLogger::init(vec![term_logger, file_logger])
        .context("Failed to install logger")
}

async fn run_cancel(cancel_rx: ChannelRx<()>) -> AnyResult<()> {
    let (signal_tx, signal_rx) = smol::channel::bounded(1);
    match ctrlc::set_handler(move || {
        let _ = signal_tx.send_blocking(());
    }) {
        Ok(()) => {
            log::debug!("Ctrl-C handler is active.");
            futures::select_biased! {
                _ = cancel_rx.recv().fuse() => {},
                _ = signal_rx.recv().fuse() => {
                    cancel_rx.close();
                },
            };
            Ok(())
        }

        Err(e) => {
            log::warn!("Failed to install Ctrl-C handler: {}", e);
            let _ = cancel_rx.recv().await;
            Ok(())
        }
    }
}
