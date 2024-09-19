use tokio::runtime::Builder;

use crate::{error::AppError, types::Operation};

mod request;
mod tui;
mod types;
mod util;

use util::*;

pub fn run_operation(operation: Option<Operation>) -> Result<(), AppError> {
    assert_service_installed()?;

    // Create async runtime to enable fetching Habitica API data
    let runtime = Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .unwrap();

    let handle = match operation {
        Some(Operation::List { save_json }) => runtime.spawn(list_tasks(save_json)),
        Some(Operation::Task { descriptor }) => runtime.spawn(create_task(descriptor)),
        Some(Operation::Reorder) => runtime.spawn(priority_reorder_tasks()),
        Some(Operation::History) => runtime.spawn(get_completed_tasks()),
        None => runtime.spawn(tui::run()),
    };

    runtime.block_on(handle)??;
    Ok(())
}
