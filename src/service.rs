use tokio::runtime::Builder;

use crate::{error::AppError, types::Operation};

mod request;
mod tui;
mod types;
mod util;

use util::*;

async fn run_async(operation: Option<Operation>) -> Result<(), AppError> {
    run_migrations().await?;

    match operation {
        Some(Operation::List { save_json }) => list_tasks(save_json).await?,
        Some(Operation::Task { descriptor }) => create_task(descriptor).await?,
        Some(Operation::Reorder) => priority_reorder_tasks().await?,
        Some(Operation::History) => get_completed_tasks().await?,
        None => tui::run().await?,
    };

    Ok(())
}

pub fn run_operation(operation: Option<Operation>) -> Result<(), AppError> {
    assert_service_installed()?;

    // Create async runtime to enable fetching Habitica API data
    let runtime = Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .unwrap();

    let handle = runtime.spawn(run_async(operation));

    runtime.block_on(handle)??;
    Ok(())
}
