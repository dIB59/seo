use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LinkType {
    Internal,
    External,
    Resource,
}

impl LinkType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Internal => "internal",
            Self::External => "external",
            Self::Resource => "resource",
        }
    }
}

impl std::str::FromStr for LinkType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "internal" => Ok(Self::Internal),
            "external" => Ok(Self::External),
            "resource" => Ok(Self::Resource),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Link {
    pub id: i64,
    pub job_id: String,
    pub source_page_id: String,
    pub target_page_id: Option<String>,

    pub target_url: String,
    pub link_text: Option<String>,
    pub link_type: LinkType,
    pub is_followed: bool,
    pub status_code: Option<i64>,
}

impl Link {
    pub fn is_external_for_url(&self, source_url: Option<&String>) -> bool {
        let Some(source_url) = source_url else {
            return !matches!(self.link_type, LinkType::Internal);
        };

        if let (Ok(source), Ok(target)) = (Url::parse(source_url), Url::parse(&self.target_url)) {
            return source.host_str() != target.host_str() || source.port() != target.port();
        }

        !matches!(self.link_type, LinkType::Internal)
    }
}

#[derive(Debug, Clone)]
pub struct NewLink {
    pub job_id: String,
    pub source_page_id: String,
    pub target_page_id: Option<String>,
    pub target_url: String,
    pub link_text: Option<String>,
    pub link_type: LinkType,
    pub is_followed: bool,
    pub status_code: Option<i64>,
}

impl NewLink {
    /// Create a new link, automatically determining if it is internal or external.
    pub fn create(
        job_id: &str,
        source_page_id: &str,
        target_url: &str,
        link_text: Option<String>,
        status_code: Option<i64>,
        current_page_url: &str,
    ) -> Self {
        let link_type = if Self::is_internal(target_url, current_page_url) {
            LinkType::Internal
        } else {
            LinkType::External
        };

        Self {
            job_id: job_id.to_string(),
            source_page_id: source_page_id.to_string(),
            target_page_id: None,
            target_url: target_url.to_string(),
            link_text,
            link_type,
            is_followed: true,
            status_code,
        }
    }

    fn is_internal(target_url: &str, base_url: &str) -> bool {
        if let (Ok(edge_url), Ok(base)) = (Url::parse(target_url), Url::parse(base_url)) {
            edge_url.host_str() == base.host_str() && edge_url.port() == base.port()
        } else {
            false
        }
    }
}
