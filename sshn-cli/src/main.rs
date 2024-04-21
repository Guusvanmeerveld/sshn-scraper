use clap::{Parser, Subcommand};
use rpassword::prompt_password;
use serde::Serialize;

use crate::commands::login::login;

mod commands;
mod error;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Login {
        /// Username of the SSHN account
        #[arg(short, long)]
        username: String,

        /// Password of the SSHN account
        #[arg(short, long)]
        password: Option<String>,

        /// The login url
        #[arg(short, long)]
        login_url: Option<String>,

        #[arg(short, long, default_value_t, value_enum)]
        webdriver: WebDriver,
    },
}

#[derive(clap::ValueEnum, Serialize, Debug, Clone, Default)]
#[serde(rename_all = "kebab-case")]
pub enum WebDriver {
    #[default]
    Chromium,
    Gecko,
}

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let args = Args::parse();

    match args.command {
        Commands::Login {
            username,
            password,
            webdriver,
            login_url,
        } => {
            let password = match password {
                Some(pass) => pass,
                None => {
                    let password = prompt_password("Enter the password of your SSHN account: ")
                        .expect("Failed to read password");

                    password
                }
            };

            log::info!("Logging in as user '{}'", username);

            let login_result = login(&username, &password, webdriver, login_url).await;

            match login_result {
                Ok(_) => {
                    log::info!("Succesfully logged in as user '{}'", username)
                }
                Err(error) => {
                    log::error!("Error logging in: {}", error);
                }
            }
        }
    }
}
