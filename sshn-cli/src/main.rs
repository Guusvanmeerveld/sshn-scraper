use clap::{Parser, Subcommand};
use rpassword::prompt_password;
use serde::Serialize;

mod auth;
mod commands;
mod error;
mod publication;
mod secrets;

use auth::AuthOptions;

/// SSHN command line interface.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Login to the SSHN API.
    Login {
        /// Username of the SSHN account.
        #[arg(short, long)]
        username: String,

        /// Password of the SSHN account.
        #[arg(short, long)]
        password: Option<String>,

        /// The login portal base url.
        #[arg(short, long)]
        login_url: Option<String>,

        /// The web driver to use to connect to the browser.
        #[arg(short, long, default_value_t, value_enum)]
        webdriver: WebDriver,
    },

    /// List the currently open publications.
    List {
        #[arg(short, long)]
        limit: Option<usize>,
    },

    /// Reply to a publication with a given id.
    Reply { id: String },
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
    {
        let mut builder = env_logger::Builder::from_default_env();

        builder.filter_module("sshn_cli", log::LevelFilter::Info);
        builder.filter_module("sshn_lib", log::LevelFilter::Info);

        builder.init();
    }

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

            match commands::login(
                &username,
                &password,
                AuthOptions::default().webdriver(webdriver),
            )
            .await
            {
                Ok(_) => {
                    log::info!("Succesfully logged in as user '{}'", username)
                }
                Err(error) => {
                    log::error!("Error logging in: {}", error);
                }
            }
        }

        Commands::List { limit } => {
            match commands::list(limit.unwrap_or(5)).await {
                Ok(_) => {}
                Err(error) => {
                    log::error!("Error listing publications: {}", error);
                }
            };
        }

        Commands::Reply { id } => {
            match commands::reply(id).await {
                Ok(_) => {}
                Err(error) => {
                    log::error!("Error replying to publication: {}", error);
                }
            };
        }
    }
}
