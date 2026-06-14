use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct BasesQuery {
    pub r#type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AccessoriesQuery {
    pub base_handle: Option<String>,
}
