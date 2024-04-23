use crate::error::{Error, Result};

pub async fn list_publications(limit: usize) -> Result<Vec<Publication>> {
    let client = sshn_lib::Client::new(None);

    let publications = client.get_publications_list(limit as i64).await?;

    Ok(publications
        .housing_publications
        .ok_or(Error::MissingPublications)?
        .nodes
        .ok_or(Error::MissingPublications)?
        .edges
        .ok_or(Error::MissingPublications)?
        .into_iter()
        .filter_map(|publication| {
            let publication = publication?.node?;

            // let city = publication.unit?.location?.city?.name.as_ref()?.to_string();
            let rent = publication.unit?.gross_rent.as_ref()?.exact;

            Some(Publication {
                id: publication.id,
                name: String::new(),
                city: String::new(),
                nr_of_applicants: publication.total_number_of_applications,
                rent,
            })
        })
        .collect())
}

pub struct Publication {
    id: String,
    name: String,
    city: String,
    nr_of_applicants: i64,
    rent: f64,
}

impl Publication {
    pub fn as_row(self) -> Vec<String> {
        vec![
            self.name,
            self.city,
            self.nr_of_applicants.to_string(),
            self.rent.to_string(),
            self.id,
        ]
    }

    pub fn row_labels() -> Vec<String> {
        vec![
            String::from("Name"),
            String::from("City"),
            String::from("Number of applicants"),
            String::from("Gross rent"),
            String::from("ID"),
        ]
    }
}
