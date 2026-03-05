use url::Url;

/// Extract the root domain from a URL string.
/// For example:
/// - "https://www.example.com" -> "example.com"
/// - "https://blog.example.com" -> "example.com"
/// - "https://sub.blog.example.com" -> "example.com"
/// - "https://example.co.uk" -> "example.co.uk"
pub fn extract_root_domain(url_str: &str) -> Option<String> {
    let url = Url::parse(url_str).ok()?;
    let host = url.host_str()?;
    
    // Remove port if present
    let host = host.split(':').next().unwrap_or(host);
    
    // Remove www. prefix
    let host = host.strip_prefix("www.").unwrap_or(host);
    
    // Extract root domain (last two parts for most domains)
    // This handles: example.com, blog.example.com, sub.blog.example.com
    let parts: Vec<&str> = host.split('.').collect();
    
    if parts.len() < 2 {
        return Some(host.to_string());
    }
    
    // Handle common TLDs that have two parts (e.g., co.uk, com.au)
    let double_tlds = [
        "co.uk", "com.au", "org.uk", "net.au", "co.nz", "co.jp",
        "com.br", "co.in", "com.sg", "co.za",
    ];
    
    let root_domain = if parts.len() >= 3 {
        let last_two = format!("{}.{}", parts[parts.len() - 2], parts[parts.len() - 1]);
        if double_tlds.contains(&last_two.as_str()) {
            // e.g., example.co.uk -> take last 3 parts
            format!(
                "{}.{}.{}",
                parts[parts.len() - 3],
                parts[parts.len() - 2],
                parts[parts.len() - 1]
            )
        } else {
            // e.g., blog.example.com -> example.com
            format!(
                "{}.{}",
                parts[parts.len() - 2],
                parts[parts.len() - 1]
            )
        }
    } else {
        // e.g., example.com
        host.to_string()
    };
    
    Some(root_domain)
}

/// Extract the host from a URL string, stripping www. prefix.
/// For example:
/// - "https://www.example.com" -> "example.com"
/// - "https://blog.example.com" -> "blog.example.com"
pub fn extract_host(url_str: &str) -> Option<String> {
    let url = Url::parse(url_str).ok()?;
    let host = url.host_str()?;
    
    // Remove www. prefix
    let host = host.strip_prefix("www.").unwrap_or(host);
    
    Some(host.to_string())
}

/// Check if two URLs share the same root domain.
/// This is useful for determining if URLs belong to the same site.
pub fn same_root_domain(url1: &str, url2: &str) -> bool {
    let domain1 = extract_root_domain(url1);
    let domain2 = extract_root_domain(url2);
    
    match (domain1, domain2) {
        (Some(d1), Some(d2)) => d1 == d2,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_root_domain_simple() {
        assert_eq!(
            extract_root_domain("https://example.com"),
            Some("example.com".to_string())
        );
    }

    #[test]
    fn test_extract_root_domain_with_www() {
        assert_eq!(
            extract_root_domain("https://www.example.com"),
            Some("example.com".to_string())
        );
    }

    #[test]
    fn test_extract_root_domain_subdomain() {
        assert_eq!(
            extract_root_domain("https://blog.example.com"),
            Some("example.com".to_string())
        );
    }

    #[test]
    fn test_extract_root_domain_deep_subdomain() {
        assert_eq!(
            extract_root_domain("https://sub.blog.example.com"),
            Some("example.com".to_string())
        );
    }

    #[test]
    fn test_extract_root_domain_double_tld() {
        assert_eq!(
            extract_root_domain("https://example.co.uk"),
            Some("example.co.uk".to_string())
        );
    }

    #[test]
    fn test_extract_root_domain_subdomain_double_tld() {
        assert_eq!(
            extract_root_domain("https://blog.example.co.uk"),
            Some("example.co.uk".to_string())
        );
    }

    #[test]
    fn test_extract_root_domain_with_port() {
        assert_eq!(
            extract_root_domain("https://example.com:8080"),
            Some("example.com".to_string())
        );
    }

    #[test]
    fn test_extract_root_domain_different_domains() {
        let google = extract_root_domain("https://www.google.com");
        let example = extract_root_domain("https://example.com");
        let test = extract_root_domain("https://sub.test.com");
        
        assert_ne!(google, example);
        assert_ne!(example, test);
    }

    #[test]
    fn test_extract_root_domain_same_domain_different_subdomains() {
        let www = extract_root_domain("https://www.example.com");
        let blog = extract_root_domain("https://blog.example.com");
        let api = extract_root_domain("https://api.example.com");
        let root = extract_root_domain("https://example.com");
        
        assert_eq!(www, root);
        assert_eq!(blog, root);
        assert_eq!(api, root);
    }

    #[test]
    fn test_extract_host() {
        assert_eq!(
            extract_host("https://www.example.com"),
            Some("example.com".to_string())
        );
        assert_eq!(
            extract_host("https://blog.example.com"),
            Some("blog.example.com".to_string())
        );
    }

    #[test]
    fn test_same_root_domain() {
        assert!(same_root_domain("https://www.example.com", "https://blog.example.com"));
        assert!(same_root_domain("https://example.com", "https://api.example.com"));
        assert!(!same_root_domain("https://example.com", "https://different.com"));
    }
}