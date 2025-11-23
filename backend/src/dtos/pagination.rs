use serde::{Deserialize, Serialize};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginationMeta {
    pub total_pages: i64,
    pub page_size: i64,
    pub total_items: i64,
}