use clap::{Parser, Subcommand};
use rpassword::read_password;
use serde::Serialize;

mod auth;
mod commands;
mod error;
mod secrets;

use auth::AuthOptions;

macro_rules! show {
    ($($arg:tt)*) => ({
        use colored::*;

        let app_name = "[SSHN-CLI]".blue().bold();

        println!("{} {}", app_name, format!($($arg)*));
    });
}

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

        /// Whether to auto start the web driver.
        #[arg(long)]
        auto_start_webdriver: bool,

        /// The port used to connect to the webdriver.
        #[arg(long)]
        webdriver_port: Option<u16>,
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
    env_logger::init();

    let args = Args::parse();

    match args.command {
        Commands::Login {
            username,
            password,
            login_url,
            webdriver,
            auto_start_webdriver,
            webdriver_port,
        } => {
            let password = match password {
                Some(pass) => pass,
                None => {
                    show!("Enter the password of your {} account: ", "SSHN".bold());
                    let password = read_password().expect("Failed to read password");

                    password
                }
            };

            show!("Logging in as user '{}'", username.bold().green());

            let mut auth_options = AuthOptions::default()
                .webdriver(webdriver)
                .auto_start_webdriver(auto_start_webdriver);

            if let Some(webdriver_port) = webdriver_port {
                auth_options = auth_options.webdriver_port(webdriver_port);
            }

            if let Some(login_url) = login_url {
                auth_options = auth_options.login_base_url(login_url);
            }

            match commands::login(&username, &password, auth_options).await {
                Ok(_) => {
                    show!(
                        "Succesfully logged in as user '{}'.",
                        username.bold().green()
                    )
                }
                Err(error) => {
                    show!("Error logging in:\n\t {}", error);
                }
            }
        }

        Commands::List { limit } => {
            match commands::list(limit.unwrap_or(5)).await {
                Ok(table) => {
                    table.printstd();
                }
                Err(error) => {
                    show!("Error listing publications:\n\t {}", error);
                }
            };
        }

        Commands::Reply { id } => {
            show!("Replying to publication...");

            match commands::reply(&id).await {
                Ok(_) => {
                    show!(
                        "Successfully replied to publication with id '{}'.",
                        id.bold().green()
                    )
                }
                Err(error) => {
                    show!("Error replying to publication:\n\t {}", error);
                }
            };
        }
    }
}
