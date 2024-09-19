use std::env;

use dotenv::dotenv;

use crate::error::AppError;

const HABITUI_CONFIG_DIR: &str = ".config/habitui";

pub fn build_config_path() -> Result<String, AppError> {
    let sudo_user_var = env::var("SUDO_USER");
    let home_var = env::var("HOME");
    let dir: String;

    match (sudo_user_var, home_var) {
        (Ok(user), _) => dir = format!("/home/{user}/{HABITUI_CONFIG_DIR}"),
        (_, Ok(home)) => dir = format!("{home}/{HABITUI_CONFIG_DIR}"),
        (Err(_), Err(e)) => return Err(e.into()),
    }

    Ok(dir)
}

pub fn load_env() -> Result<(), AppError> {
    let cwd = env::current_dir()?;
    let dir = build_config_path()?;

    // Go to config dir and pull .env contents
    if let Err(_) = env::set_current_dir(dir) {
        return Err(AppError::ServiceError(
            "$HOME/.config/habitui not found".to_string(),
        ));
    }

    dotenv().ok();

    env::set_current_dir(cwd)?;
    Ok(())
}
