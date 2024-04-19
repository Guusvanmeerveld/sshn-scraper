use graphql_client::GraphQLQuery;
use serde::Deserialize;

type Cursor = String;
type Decimal = f64;
type DateTimeOffset = String;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries.graphql",
    response_derives = "Debug"
)]
pub struct GetPublicationsList;

#[derive(Deserialize)]
pub struct GraphqlResponse<T> {
    pub data: T,
}
