use url::Url;

const SEARCH_BASE: &str = "https://duckduckgo.com/";

pub(crate) fn normalize_url(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    if trimmed.contains("://") || has_special_scheme(trimmed) {
        return trimmed.to_string();
    }

    if looks_like_host(trimmed) {
        let scheme = if is_loopback(trimmed) {
            "http"
        } else {
            "https"
        };
        let candidate = format!("{scheme}://{trimmed}");
        if Url::parse(&candidate).is_ok() {
            return candidate;
        }
    }

    search_url(trimmed)
}

fn has_special_scheme(s: &str) -> bool {
    const SCHEMES: &[&str] = &["about:", "data:", "javascript:", "mailto:"];
    SCHEMES.iter().any(|p| s.starts_with(p))
}

fn search_url(query: &str) -> String {
    Url::parse_with_params(SEARCH_BASE, &[("q", query)])
        .map(|u| u.to_string())
        .unwrap_or_else(|_| format!("{SEARCH_BASE}?q={query}"))
}

fn looks_like_host(s: &str) -> bool {
    !s.chars().any(char::is_whitespace) && (s.contains('.') || is_loopback(s) || s.starts_with('['))
}

fn is_loopback(s: &str) -> bool {
    is_host_prefix(s, "localhost") || is_host_prefix(s, "127.0.0.1") || s.starts_with("[::1]")
}

fn is_host_prefix(s: &str, host: &str) -> bool {
    match s.strip_prefix(host) {
        None => false,
        Some("") => true,
        Some(rest) => matches!(rest.as_bytes()[0], b':' | b'/' | b'?' | b'#'),
    }
}

#[cfg(test)]
mod tests {
    use super::normalize_url;

    #[test]
    fn passes_through_explicit_scheme() {
        assert_eq!(normalize_url("https://example.com"), "https://example.com");
        assert_eq!(normalize_url("http://example.com"), "http://example.com");
        assert_eq!(normalize_url("file:///tmp/a.html"), "file:///tmp/a.html");
    }

    #[test]
    fn passes_through_special_schemes() {
        assert_eq!(normalize_url("about:blank"), "about:blank");
        assert_eq!(normalize_url("data:text/plain,hi"), "data:text/plain,hi");
        assert_eq!(normalize_url("javascript:void(0)"), "javascript:void(0)");
    }

    #[test]
    fn prepends_https_for_bare_host() {
        assert_eq!(normalize_url("example.com"), "https://example.com");
        assert_eq!(
            normalize_url("en.wikipedia.org/wiki/X"),
            "https://en.wikipedia.org/wiki/X"
        );
    }

    #[test]
    fn uses_http_for_loopback() {
        assert_eq!(normalize_url("localhost"), "http://localhost");
        assert_eq!(normalize_url("localhost:8080"), "http://localhost:8080");
        assert_eq!(normalize_url("127.0.0.1:3000"), "http://127.0.0.1:3000");
        assert_eq!(normalize_url("[::1]:8080"), "http://[::1]:8080");
    }

    #[test]
    fn loopback_lookalike_uses_https() {
        assert_eq!(
            normalize_url("localhostfoo.com"),
            "https://localhostfoo.com"
        );
    }

    #[test]
    fn trims_whitespace() {
        assert_eq!(normalize_url("  example.com  "), "https://example.com");
    }

    #[test]
    fn empty_returns_empty() {
        assert_eq!(normalize_url(""), "");
        assert_eq!(normalize_url("   "), "");
    }

    #[test]
    fn bare_word_searches() {
        let out = normalize_url("rust");
        assert!(out.starts_with("https://duckduckgo.com/"));
        assert!(out.contains("q=rust"));
    }

    #[test]
    fn whitespace_input_searches() {
        let out = normalize_url("hello world");
        assert!(out.starts_with("https://duckduckgo.com/"));
        assert!(out.contains("q=hello") && out.contains("world"));
    }

    #[test]
    fn question_input_searches() {
        let out = normalize_url("what is rust?");
        assert!(out.starts_with("https://duckduckgo.com/"));
        assert!(out.contains("q=what"));
    }
}
