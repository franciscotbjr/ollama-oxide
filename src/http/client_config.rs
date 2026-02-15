//! HTTP client configuration

use std::time::Duration;

/// Configuration for Ollama HTTP client
///
/// This struct allows customization of the HTTP client behavior including
/// base URL, timeout, and retry settings.
///
/// # Examples
///
/// ```no_run
/// use ollama_oxide::ClientConfig;
/// use std::time::Duration;
///
/// // Use default configuration
/// let config = ClientConfig::default();
///
/// // Custom configuration
/// let config = ClientConfig {
///     base_url: "http://example.com:8080".to_string(),
///     timeout: Duration::from_secs(60),
///     max_retries: 5,
/// };
/// ```
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Base URL for Ollama API
    ///
    /// Must include the scheme (http:// or https://)
    pub base_url: String,

    /// Request timeout duration
    ///
    /// How long to wait for a response before timing out
    pub timeout: Duration,

    /// Maximum retry attempts on failure
    ///
    /// Number of times to retry a failed request (0 = no retries)
    pub max_retries: u32,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:11434".to_string(),
            timeout: Duration::from_secs(30),
            max_retries: 3,
        }
    }
}

impl ClientConfig {
    /// Creates a new `ClientConfig` with all attributes specified.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ollama_oxide::ClientConfig;
    /// use std::time::Duration;
    ///
    /// let config = ClientConfig::new(
    ///     "http://example.com:8080".to_string(),
    ///     Duration::from_secs(60),
    ///     5,
    /// );
    /// ```
    pub fn new(base_url: String, timeout: Duration, max_retries: u32) -> Self {
        Self {
            base_url,
            timeout,
            max_retries,
        }
    }

    /// Creates a new `ClientConfig` with only `base_url`, using defaults for `timeout` (30s) and `max_retries` (3).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ollama_oxide::ClientConfig;
    ///
    /// let config = ClientConfig::with_base_url("http://example.com:8080".to_string());
    /// assert_eq!(config.timeout, std::time::Duration::from_secs(30));
    /// assert_eq!(config.max_retries, 3);
    /// ```
    pub fn with_base_url(base_url: String) -> Self {
        Self {
            base_url,
            ..Self::default()
        }
    }

    /// Creates a new `ClientConfig` with `base_url` and `timeout`, using the default `max_retries` (3).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ollama_oxide::ClientConfig;
    /// use std::time::Duration;
    ///
    /// let config = ClientConfig::with_base_url_and_timeout(
    ///     "http://example.com:8080".to_string(),
    ///     Duration::from_secs(60),
    /// );
    /// assert_eq!(config.max_retries, 3);
    /// ```
    pub fn with_base_url_and_timeout(base_url: String, timeout: Duration) -> Self {
        Self {
            base_url,
            timeout,
            ..Self::default()
        }
    }

    /// Build full URL from base URL and endpoint path
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ollama_oxide::ClientConfig;
    ///
    /// let config = ClientConfig::default();
    /// let url = config.url("/api/version");
    /// assert_eq!(url, "http://localhost:11434/api/version");
    /// ```
    #[inline]
    pub fn url(&self, endpoint: &str) -> String {
        format!("{}{}", self.base_url, endpoint)
    }
}
