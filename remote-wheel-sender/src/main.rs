use std::{process::{ExitCode}, sync::Arc};

use anyhow::{Context as _, Result as AnyResult, ensure};
use futures::prelude::*;

mod config;
mod input;
mod output;

fn main() -> ExitCode {
    init_logger().expect("Failed to initialize logging");

    match run_sync() {
        Ok(()) => {
            log::debug!("Clean exit.");
            ExitCode::SUCCESS
        },

        Err(e) => {
            log::error!("An error has occurred: {:#}", e);
            log::logger().flush();

            eprintln!("Press any key to exit.");
            let _ = console::Term::stdout().read_key();

            ExitCode::FAILURE
        },
    }
}

fn run_sync() -> AnyResult<()> {
   smol::block_on(run_async())
}

async fn run_async() -> AnyResult<()> {
    let config = config::AppConfig::read_from("remote-wheel-sender.yaml").await?;
    config.validate()
        .context("Failed to validate configuration")?;

    let (output_tx, output_rx) = async_broadcast::broadcast(16);
    let (cancel_tx, cancel_rx) = smol::channel::unbounded();

    let exec = Arc::new(smol::Executor::new());
    let mut tasks = Vec::new();

    #[cfg(feature = "osc")]
    {
        let osc_task = exec.spawn(crate::output::osc::run(config.osc, output_rx));
        tasks.push(osc_task);
    }

    ensure!(!exec.is_empty(), "No outputs are enabled!");
    let input_task = exec.spawn(crate::input::run(exec.clone(), config.inputs, output_tx, cancel_rx.clone()));
    tasks.push(input_task);

    {
        let cancel_rx = cancel_rx.clone();
        let cancel_task = exec.spawn(async move {
            let (signal_tx, signal_rx) = smol::channel::bounded(1);
            match ctrlc::set_handler(move || {
                let _ = signal_tx.send_blocking(());
            }) {
                Ok(()) => {
                    futures::select_biased!{
                        _ = cancel_rx.recv().fuse() => {},
                        _ = signal_rx.recv().fuse() => {},
                    };
                    Ok(())
                },

                Err(e) => {
                    log::warn!("Failed to install Ctrl-C handler: {}", e);
                    let _ = cancel_rx.recv().await;
                    Ok(())
                },
            }
        });
        tasks.push(cancel_task);
    }

    exec.run(async {
        let mut result = Ok(());

        while !tasks.is_empty() {
            let (task_result, _, rest) = future::select_all(tasks).await;
            cancel_tx.close();

            result = result.and(task_result);
            tasks = rest;
        }

        result
    }).await
}

fn init_logger() -> AnyResult<()> {
    let term_config = simplelog::ConfigBuilder::new()
        .set_time_offset_to_local().unwrap_or_else(|e| e)
        .build();

    let term_logger = simplelog::TermLogger::new(
        log::LevelFilter::Info,
        term_config,
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Auto);

    let file = std::fs::File::options()
        .create(true)
        .truncate(true)
        .write(true)
        .open("remote-wheel-sender.log")
        .context("Failed to open log file")?;

    let file_config = simplelog::ConfigBuilder::new()
        .set_time_format_custom(simplelog::format_description!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]"))
        .set_time_offset_to_local().unwrap_or_else(|e| e)
        .build();

    let file_logger = simplelog::WriteLogger::new(
        log::LevelFilter::Trace,
        file_config,
        file);

    simplelog::CombinedLogger::init(vec![term_logger, file_logger])
        .context("Failed to install logger")
}
