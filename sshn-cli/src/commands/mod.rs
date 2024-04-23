use crate::{
    auth::{self, AuthOptions},
    error::Result,
    publication::{self, Publication},
    secrets,
};

pub async fn login<U: AsRef<str>, P: AsRef<str>>(
    username: U,
    password: P,
    options: AuthOptions,
) -> Result<()> {
    auth::password_login(username.as_ref(), password.as_ref(), options).await?;

    Ok(())
}

pub async fn list(limit: usize) -> Result<()> {
    let data = publication::list_publications(limit).await?;

    let mut table = prettytable::Table::new();

    table.add_row(prettytable::Row::new(
        Publication::row_labels()
            .iter()
            .map(|label| prettytable::Cell::new(label))
            .collect(),
    ));

    for publication in data {
        table.add_row(prettytable::Row::new(
            publication
                .as_row()
                .iter()
                .map(|label| prettytable::Cell::new(label))
                .collect(),
        ));
    }

    table.printstd();

    Ok(())
}

pub async fn reply<I: AsRef<str>>(id: I) -> Result<()> {
    let mut client = secrets::get_client().await?;

    client.reply_to_publication(id.as_ref()).await?;

    Ok(())
}
