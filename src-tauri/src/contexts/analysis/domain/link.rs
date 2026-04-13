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

/// Returned by [`LinkType::from_str`] when the input doesn't map to a known
/// link type. Carries the offending string so decoder errors are diagnosable.
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
#[error("invalid link type: '{0}'")]
pub struct ParseLinkTypeError(pub String);

impl std::str::FromStr for LinkType {
    type Err = ParseLinkTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "internal" => Ok(Self::Internal),
            "subdomain" => Ok(Self::Subdomain),
            "external" => Ok(Self::External),
            "resource" => Ok(Self::Resource),
            other => Err(ParseLinkTypeError(other.to_string())),
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

        // Parse each URL exactly once and pull both host and port from
        // the parsed value. The previous version called `extract_host`
        // (which parses internally) and then `Url::parse` again for the
        // port — 4 parses per call. On a 1000-link page that was 4000
        // wasted parses.
        let parsed_src = Url::parse(source_url);
        let parsed_tgt = Url::parse(&self.target_url);

        if let (Ok(src), Ok(tgt)) = (parsed_src.as_ref(), parsed_tgt.as_ref()) {
            let src_host = src.host_str().map(strip_www);
            let tgt_host = tgt.host_str().map(strip_www);
            if let (Some(s), Some(t)) = (src_host, tgt_host) {
                return s != t || src.port() != tgt.port();
            }
        }

        !matches!(self.link_type, LinkType::Internal)
    }
}

/// Strip a leading `www.` from a host string for canonical comparison.
fn strip_www(host: &str) -> &str {
    host.strip_prefix("www.").unwrap_or(host)
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
        Self::classify_urls(&target, &base)
    }

    /// Classify a link given already-parsed URLs. Use this from any
    /// call site that already holds `Url` values to avoid the redundant
    /// re-parse `classify` would otherwise do.
    pub fn classify_urls(target: &Url, base: &Url) -> LinkType {
        if target.scheme() != "http" && target.scheme() != "https" {
            return LinkType::Resource;
        }

        let (Some(target_host), Some(base_host)) = (target.host_str(), base.host_str()) else {
            return LinkType::External;
        };
        let target_host = strip_www(target_host);
        let base_host = strip_www(base_host);

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
