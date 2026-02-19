use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceStatus {
    #[default]
    NotChecked,
    Found(String),
    NotFound,
    Unauthorized(String),
    Error,
}

impl ResourceStatus {
    /// Returns true if the resource exists (Found or Unauthorized)
    pub fn exists(&self) -> bool {
        matches!(self, Self::Found(_) | Self::Unauthorized(_))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Heading {
    pub id: i64,
    pub page_id: String,
    pub level: i64, // 1-6
    pub text: String,
    pub position: i64,
}

#[derive(Debug, Clone)]
pub struct NewHeading {
    pub page_id: String,
    pub level: i64,
    pub text: String,
    pub position: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Image {
    pub id: i64,
    pub page_id: String,
    pub src: String,
    pub alt: Option<String>,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub loading: Option<String>,
    pub is_decorative: bool,
}

#[derive(Debug, Clone)]
pub struct NewImage {
    pub page_id: String,
    pub src: String,
    pub alt: Option<String>,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub loading: Option<String>,
    pub is_decorative: bool,
}
