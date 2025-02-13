use sshn_lib::Client;

use crate::{
    auth::{self, AuthOptions},
    error::Result,
    secrets,
};

pub async fn login<U: AsRef<str>, P: AsRef<str>>(
    username: U,
    password: P,
    options: AuthOptions,
) -> Result<()> {
    auth::headless_login(username.as_ref(), password.as_ref(), options).await?;

    Ok(())
}

pub async fn list(limit: usize) -> Result<prettytable::Table> {
    use prettytable::{Cell, Row, Table};

    let limit = limit as i64;

    let missing_credentials = secrets::get::<_, secrets::Credentials>("credentials").is_err();

    let publications = if missing_credentials {
        let mut client = sshn_lib::UnAuthenticatedClient::new(None);

        client.get_publications_list(limit).await?
    } else {
        let mut client = secrets::get_client().await?;

        client.get_publications_list(limit).await?
    };

    let mut table = Table::new();

    table.add_row(Row::new(vec![
        Cell::new("Can reply?"),
        Cell::new("Name"),
        Cell::new("City"),
        Cell::new("Number of applicants"),
        Cell::new("Gross rent"),
        Cell::new("ID"),
    ]));

    for publication in publications {
        let nr_of_applicants_string = publication.nr_of_applicants().to_string();
        let gross_rent_string = publication.rent().to_string();
        let is_match = if publication.is_match() { "Yes" } else { "No" };

        table.add_row(Row::new(vec![
            Cell::new(is_match),
            Cell::new(publication.name()),
            Cell::new(publication.city()),
            Cell::new(&nr_of_applicants_string),
            Cell::new(&gross_rent_string),
            Cell::new(publication.id()),
        ]));
    }

    Ok(table)
}

pub async fn reply<I: AsRef<str>>(id: I) -> Result<()> {
    let mut client = secrets::get_client().await?;

    client.reply_to_publication(id.as_ref()).await?;

    Ok(())
}
