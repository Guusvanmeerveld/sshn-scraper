use serde::{Deserialize, Serialize};

use crate::{
    error::{Error, Result},
    queries::get_publications_list,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct Publication {
    id: String,
    name: String,
    city: String,
    nr_of_applicants: i64,
    nr_of_people_with_higher_priority: i64,
    is_match: bool,
    rent: f64,
}

impl Publication {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn city(&self) -> &str {
        &self.city
    }

    pub fn nr_of_applicants(&self) -> i64 {
        self.nr_of_applicants
    }

    pub fn nr_of_people_with_higher_priority(&self) -> i64 {
        self.nr_of_people_with_higher_priority
    }

    pub fn is_match(&self) -> bool {
        self.is_match
    }

    pub fn rent(&self) -> f64 {
        self.rent
    }
}

pub fn convert_publications(data: get_publications_list::ResponseData) -> Result<Vec<Publication>> {
    Ok(data
        .housing_publications
        .ok_or(Error::MissingPublications)?
        .nodes
        .ok_or(Error::MissingPublications)?
        .edges
        .ok_or(Error::MissingPublications)?
        .into_iter()
        .filter_map(|publication| {
            let publication = publication?.node?;

            let unit = publication.unit.as_ref()?;
            let name = unit.complex_type.as_ref()?.name.as_ref()?.to_string();
            let city = unit
                .location
                .as_ref()?
                .city
                .as_ref()?
                .name
                .as_ref()?
                .to_string();
            let rent = unit.gross_rent.as_ref()?.exact;

            Some(Publication {
                is_match: publication.applicant_specific.as_ref()?.is100_percent_match,
                id: publication.id,
                name,
                city,
                nr_of_applicants: publication.total_number_of_applications,
                nr_of_people_with_higher_priority: publication
                    .applicant_specific
                    .as_ref()?
                    .number_of_applicants_with_higher_priority
                    .unwrap_or(0),
                rent,
            })
        })
        .collect())
}
