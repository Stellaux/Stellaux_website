use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct AddItem {
    pub variant_id: Uuid,
    pub qty: i32,
    pub assembly_id: Option<Uuid>,
    pub assembly_role: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateItem {
    pub qty: i32,
}
