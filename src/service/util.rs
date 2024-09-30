use std::env;
use std::fs::File;
use std::io::Write;

use inquire::{max_length, min_length, DateSelect, Select, Text};
use log::debug;
use serde::{Deserialize, Serialize};
use sqlx::{types::uuid::Uuid, PgPool, Postgres};
use time::{format_description::well_known::Iso8601, OffsetDateTime};

use super::{
    request::{fetch_tasks, post_created_task, reorder_task},
    types::{Difficulty, Priority, SubTask, Task},
};
use crate::{error::AppError, util::build_config_path};

pub const ISO8601: Iso8601 = Iso8601::DEFAULT;

#[derive(Serialize, Deserialize)]
pub struct ArrayRes<T> {
    pub data: Vec<T>,
}
#[derive(Serialize, Deserialize)]
pub struct SingleRes<T> {
    pub data: T,
}

pub fn get_json_path() -> Result<String, AppError> {
    let dir = build_config_path()?;
    Ok(format!("{dir}/habitica_tasks.json"))
}

pub fn get_env_vars() -> Result<(String, String, String, String), AppError> {
    Ok((
        env::var("HABITICA_USER_ID")?,
        env::var("HABITICA_TOKEN")?,
        env::var("HABITICA_XCLIENT")?,
        env::var("POSTGRES_URL")?,
    ))
}

pub fn assert_service_installed() -> Result<(), AppError> {
    // Test that env was loaded correctly
    get_env_vars()?;

    Ok(())
}

pub async fn create_pg_pool() -> Result<PgPool, AppError> {
    let pool = PgPool::connect(&env::var("POSTGRES_URL")?).await?;
    Ok(pool)
}

pub async fn run_migrations() -> Result<(), AppError> {
    let pool = create_pg_pool().await?;
    sqlx::migrate!().run(&pool).await?;

    Ok(())
}

fn parse_difficulty(selected: &str) -> Result<Difficulty, AppError> {
    let parsed: Difficulty = match selected {
        "Trivial" => Difficulty::TRIVIAL,
        "Easy" => Difficulty::EASY,
        "Medium" => Difficulty::MEDIUM,
        "Hard" => Difficulty::HARD,
        _ => Err(AppError::CmdError("Incorrect difficulty value".into()))?,
    };

    Ok(parsed)
}

fn parse_task_descriptor(descriptor: String) -> Result<Task, AppError> {
    let mut parts = descriptor.split(",");
    let parts = (
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
    );
    match parts {
        (Some(text), Some(difficulty), notes, date, check) => {
            return Ok(Task {
                id: Uuid::nil(),
                text: text.into(),
                task_type: "todo".into(),
                difficulty: parse_difficulty(difficulty)?,
                notes: notes.map(|n| n.into()),
                date: date.map(|d| OffsetDateTime::parse(d.into(), &ISO8601).unwrap()),
                completed_at: None,
                checklist: check.map(|c| {
                    c.split(";")
                        .map(|i| SubTask {
                            text: i.into(),
                            completed: false,
                        })
                        .collect()
                }),
            });
        }
        (None, ..) => Err(AppError::CmdError(
            "Incorrect input: <name> required".into(),
        ))?,
        (_, None, ..) => Err(AppError::CmdError(
            "Incorrect input: <difficulty> required".into(),
        ))?,
    }
}

fn checklist_item_formatter(i: &str) -> String {
    format!("[] {i}")
}

fn prompt_for_checklist() -> Result<Option<Vec<SubTask>>, AppError> {
    let mut list: Vec<SubTask> = Vec::new();
    let mut finished = false;
    let mut i = 1;

    while !finished {
        let item = Text::new(format!("Checlist item #{i}:").as_str())
            .with_help_message("Press ESC to skip")
            .with_formatter(&checklist_item_formatter)
            .prompt_skippable()?;

        if item.is_none() {
            finished = true;
        } else {
            list.push(SubTask {
                text: item.unwrap(),
                completed: false,
            })
        }

        i += 1;
    }

    Ok(if list.is_empty() { None } else { Some(list) })
}

fn prompt_for_task() -> Result<Task, AppError> {
    let name = Text::new("Task name:")
        .with_validator(min_length!(1, "Task name cannot be empty."))
        .with_validator(max_length!(60, "Task name must be at most 60 characters."))
        .prompt()?;

    let difficulty = Select::new(
        "Difficulty:",
        vec![
            Difficulty::TRIVIAL,
            Difficulty::EASY,
            Difficulty::MEDIUM,
            Difficulty::HARD,
        ],
    )
    .with_vim_mode(true)
    .prompt()?;

    let notes = Text::new("Extra notes:")
        .with_validator(max_length!(60, "Notes must be at most 60 characters."))
        .prompt()?;

    let date = DateSelect::new("Due date:")
        .with_help_message("Press ESC to skip")
        .prompt_skippable()?
        .map(|d| OffsetDateTime::parse(&d.format("%F").to_string(), &ISO8601).unwrap());

    let checklist = prompt_for_checklist()?;

    Ok(Task {
        id: Uuid::nil(),
        text: name,
        task_type: "todo".into(),
        difficulty,
        notes: if notes.is_empty() { None } else { Some(notes) },
        date,
        completed_at: None,
        checklist,
    })
}

async fn query_completed_tasks() -> Result<Vec<Task>, AppError> {
    let pool = create_pg_pool().await?;
    let tasks = sqlx::query_as::<_, Task>(
        "SELECT
            id,
            text,
            task_type,
            difficulty,
            notes,
            date,
            completed_at,
            checklist 
        FROM completed_task;
        ",
    )
    .fetch_all(&pool)
    .await?;

    Ok(tasks)
}

pub async fn create_task(descriptor: Option<String>) -> Result<(), AppError> {
    let task: Task;
    if descriptor.is_some() {
        task = parse_task_descriptor(descriptor.unwrap())?;
    } else {
        task = prompt_for_task()?;
    }
    debug!("Creating task: \n{task}");

    let created = post_created_task(task).await?;

    println!("Created: \n{}", created);

    Ok(())
}

pub async fn get_task_list() -> Result<Vec<Task>, AppError> {
    let raw_tasks = fetch_tasks("todos").await?;
    let tasks = serde_json::from_str::<ArrayRes<Task>>(raw_tasks.as_str())?.data;
    Ok(tasks)
}

pub async fn list_tasks(save_json: bool) -> Result<(), AppError> {
    let raw_tasks = fetch_tasks("todos").await?;
    let tasks = serde_json::from_str::<ArrayRes<Task>>(raw_tasks.as_str())?.data;

    for task in tasks {
        println!("{task}");
    }

    if save_json {
        let mut file = File::create(get_json_path()?)?;
        file.write_all(raw_tasks.as_bytes())?;
        println!("\nSaved list to ~/.config/habitica_tasks.json");
    }

    Ok(())
}

pub async fn get_completed_tasks() -> Result<(), AppError> {
    let raw_tasks = fetch_tasks("completedTodos").await?;
    let tasks = serde_json::from_str::<ArrayRes<Task>>(raw_tasks.as_str())?.data;

    let pool = PgPool::connect(&env::var("POSTGRES_URL")?).await?;

    for task in tasks {
        sqlx::query::<Postgres>(
            "
            INSERT INTO completed_task (
                id,
                text,
                task_type,
                difficulty,
                notes,
                date,
                completed_at,
                checklist
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT(id) DO NOTHING;
            ",
        )
        .bind(task.id)
        .bind(task.text)
        .bind(task.task_type)
        .bind(task.difficulty)
        .bind(task.notes)
        .bind(task.date)
        .bind(task.completed_at)
        .bind(task.checklist)
        .execute(&pool)
        .await?;
    }

    for task in query_completed_tasks().await? {
        println!("{task}");
    }

    Ok(())
}

pub async fn priority_reorder_tasks() -> Result<(), AppError> {
    let tasks = get_task_list().await?;
    let mut prev_high_priority = 0;
    let mut prev_mid_priority = 0;

    for task in tasks {
        let prio = task.get_priority();
        match prio {
            Priority::LOW => {}
            Priority::MID => {
                reorder_task(task.id, prev_mid_priority).await?;
                prev_mid_priority += 1;
            }
            Priority::HIGH => {
                reorder_task(task.id, prev_high_priority).await?;
                prev_high_priority += 1;
                prev_mid_priority += 1;
            }
        }
    }

    Ok(())
}
