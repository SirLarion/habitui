use reqwest as req;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use sqlx::types::uuid::Uuid;

use crate::{
    error::AppError,
    service::{
        types::Task,
        util::{get_env_vars, SingleRes},
    },
};

const HABITICA_API_ENDPOINT: &str = "https://habitica.com/api/v3";

fn get_headers() -> Result<HeaderMap, AppError> {
    let (id, token, xclient, _) = get_env_vars()?;

    let mut headers = HeaderMap::new();
    headers.insert("x-api-user", HeaderValue::from_str(id.as_str())?);
    headers.insert("x-api-key", HeaderValue::from_str(token.as_str())?);
    headers.insert("x-client", HeaderValue::from_str(xclient.as_str())?);
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    Ok(headers)
}

pub async fn post_created_task(task: Task) -> Result<Task, AppError> {
    let client = req::Client::new();
    let headers = get_headers()?;
    let res = client
        .post(format!("{HABITICA_API_ENDPOINT}/tasks/user"))
        .json::<Task>(&task)
        .headers(headers)
        .send()
        .await?
        .error_for_status()?;

    let created = serde_json::from_str::<SingleRes<Task>>(&res.text().await?)?;

    Ok(created.data)
}

pub async fn edit_task(task: &Task) -> Result<Task, AppError> {
    let client = req::Client::new();
    let headers = get_headers()?;
    let res = client
        .put(format!("{HABITICA_API_ENDPOINT}/tasks/{}", task.id))
        .json::<Task>(task)
        .headers(headers)
        .send()
        .await?
        .error_for_status()?;

    let created = serde_json::from_str::<SingleRes<Task>>(&res.text().await?)?;
    Ok(created.data)
}

pub async fn remove_task(task_id: Uuid) -> Result<Task, AppError> {
    let client = req::Client::new();
    let headers = get_headers()?;
    let res = client
        .delete(format!("{HABITICA_API_ENDPOINT}/tasks/{}", task_id))
        .headers(headers)
        .send()
        .await?
        .error_for_status()?;

    let removed = serde_json::from_str::<SingleRes<Task>>(&res.text().await?)?;
    Ok(removed.data)
}

pub async fn complete_task(task_id: Uuid) -> Result<(), AppError> {
    let client = req::Client::new();
    let headers = get_headers()?;

    client
        .post(format!(
            "{HABITICA_API_ENDPOINT}/tasks/{}/score/up",
            task_id
        ))
        .headers(headers)
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}

pub async fn reorder_task(task_id: Uuid, index: usize) -> Result<(), AppError> {
    let client = req::Client::new();
    let headers = get_headers()?;

    client
        .post(format!(
            "{HABITICA_API_ENDPOINT}/tasks/{}/move/to/{}",
            task_id, index
        ))
        .headers(headers)
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}

/// Fetch all tasks of type <task_type> from Habitica API. For our purposes a "todo"
/// task is the same as a task in general
pub async fn fetch_tasks(task_type: &str) -> Result<String, AppError> {
    if !["todos", "completedTodos"].contains(&task_type) {
        Err(AppError::ServiceError(format!(
            "Undefined task type: {task_type}"
        )))?;
    }
    let client = req::Client::new();
    let headers = get_headers()?;
    let res = client
        .get(format!(
            "{HABITICA_API_ENDPOINT}/tasks/user?type={task_type}"
        ))
        .headers(headers)
        .send()
        .await?;

    Ok(res.text().await?)
}
