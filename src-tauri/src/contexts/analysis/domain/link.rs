use super::url_utils::extract_host;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "lowercase")]
pub enum LinkType {
    Internal,
    Subdomain,
    External,
    Resource,
}

impl LinkType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Internal => "internal",
            Self::Subdomain => "subdomain",
            Self::External => "external",
            Self::Resource => "resource",
        }
    }

    pub fn should_follow(&self, include_subdomains: bool) -> bool {
        match self {
            Self::Internal => true,
            Self::Subdomain => include_subdomains,
            Self::External => false,
            Self::Resource => false,
        }
    }
}

impl std::str::FromStr for LinkType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "internal" => Ok(Self::Internal),
            "subdomain" => Ok(Self::Subdomain),
            "external" => Ok(Self::External),
            "resource" => Ok(Self::Resource),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    pub id: String,
    pub job_id: String,
    pub source_page_id: String,
    pub target_url: String,
    pub link_text: Option<String>,
    pub link_type: LinkType,
    pub status_code: Option<i64>,
}

impl Link {
    /// Returns true if the target URL is considered "external" relative to the source URL.
    /// Strips 'www.' prefix for consistent comparison.
    pub fn is_external_for_url(&self, source_url: &str) -> bool {
        if self.target_url == source_url {
            return false;
        };

        // Use centralized host extraction
        let source_host = extract_host(source_url);
        let target_host = extract_host(&self.target_url);

        if let (Some(source), Some(target)) = (source_host, target_host) {
            // Also check port for complete comparison
            let source_port = Url::parse(source_url).ok().and_then(|u| u.port());
            let target_port = Url::parse(&self.target_url).ok().and_then(|u| u.port());
            return source != target || source_port != target_port;
        }

        !matches!(self.link_type, LinkType::Internal)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewLink {
    pub job_id: String,
    pub source_page_id: String,
    pub target_url: String,
    pub link_text: Option<String>,
    pub status_code: Option<i64>,
    pub link_type: LinkType,
}

impl NewLink {
    /// Create a new link, automatically determining if it is internal, subdomain, or external.
    pub fn create(
        job_id: &str,
        source_page_id: &str,
        target_url: &str,
        link_text: Option<String>,
        status_code: Option<i64>,
        current_page_url: &str,
    ) -> Self {
        let link_type = Self::classify(target_url, current_page_url);

        Self {
            job_id: job_id.to_string(),
            source_page_id: source_page_id.to_string(),
            target_url: target_url.to_string(),
            link_text,
            status_code,
            link_type,
        }
    }

    pub fn classify(target_url: &str, base_url: &str) -> LinkType {
        let (Ok(target), Ok(base)) = (Url::parse(target_url), Url::parse(base_url)) else {
            return LinkType::External;
        };

        if target.scheme() != "http" && target.scheme() != "https" {
            return LinkType::Resource;
        }

        // Use centralized host extraction
        let target_host = extract_host(target_url);
        let base_host = extract_host(base_url);

        let (Some(target_host), Some(base_host)) = (target_host, base_host) else {
            return LinkType::External;
        };

        if target_host == base_host && target.port() == base.port() {
            return LinkType::Internal;
        }

        // Check if one is a subdomain of the other
        let target_as_subdomain = format!(".{}", base_host);
        let base_as_subdomain = format!(".{}", target_host);

        if target_host.ends_with(&target_as_subdomain) || base_host.ends_with(&base_as_subdomain) {
            return LinkType::Subdomain;
        }

        LinkType::External
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_subdomain() {
        assert_eq!(
            NewLink::classify("https://sub.example.com", "https://example.com").as_str(),
            "subdomain"
        );
        assert_eq!(
            NewLink::classify("https://example.com", "https://sub.example.com").as_str(),
            "subdomain"
        );
        assert_eq!(
            NewLink::classify("https://sub.localhost", "http://localhost").as_str(),
            "subdomain"
        );
        assert_eq!(
            NewLink::classify("https://blog.example.co.uk", "https://example.co.uk").as_str(),
            "subdomain"
        );
    }
}
