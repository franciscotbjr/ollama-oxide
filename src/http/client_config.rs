//! HTTP client configuration

use std::time::Duration;

use crate::{Error, Result};
use url::Url;

/// Validates that a URL is well-formed and uses http or https scheme
fn validate_base_url(base_url: &str) -> Result<()> {
    let url = Url::parse(base_url)?;
    if url.scheme() != "http" && url.scheme() != "https" {
        return Err(Error::InvalidUrlError(
            url::ParseError::RelativeUrlWithoutBase,
        ));
    }
    Ok(())
}

/// Configuration for Ollama HTTP client
///
/// This struct allows customization of the HTTP client behavior including
/// base URL, timeout, and retry settings. All constructors validate that
/// the base URL is well-formed and uses http or https scheme.
///
/// # Examples
///
/// ```no_run
/// use ollama_oxide::ClientConfig;
/// use std::time::Duration;
///
/// // Use default configuration (http://localhost:11434)
/// let config = ClientConfig::default();
///
/// // Custom configuration
/// let config = ClientConfig::new(
///     "http://example.com:8080".to_string(),
///     Duration::from_secs(60),
///     5,
/// )?;
///
/// // Just a custom URL
/// let config = ClientConfig::with_base_url("http://example.com:8080".to_string())?;
/// # Ok::<(), ollama_oxide::Error>(())
/// ```
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Base URL for Ollama API (validated: must be http or https)
    base_url: String,

    /// Request timeout duration
    timeout: Duration,

    /// Maximum retry attempts on failure (0 = no retries)
    max_retries: u32,
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
    /// # Errors
    ///
    /// Returns an error if the base URL is invalid or uses an unsupported scheme.
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
    /// )?;
    /// # Ok::<(), ollama_oxide::Error>(())
    /// ```
    pub fn new(base_url: String, timeout: Duration, max_retries: u32) -> Result<Self> {
        validate_base_url(&base_url)?;
        Ok(Self {
            base_url,
            timeout,
            max_retries,
        })
    }

    /// Creates a new `ClientConfig` with only `base_url`, using defaults for `timeout` (30s) and `max_retries` (3).
    ///
    /// # Errors
    ///
    /// Returns an error if the base URL is invalid or uses an unsupported scheme.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ollama_oxide::ClientConfig;
    ///
    /// let config = ClientConfig::with_base_url("http://example.com:8080".to_string())?;
    /// assert_eq!(config.timeout(), std::time::Duration::from_secs(30));
    /// assert_eq!(config.max_retries(), 3);
    /// # Ok::<(), ollama_oxide::Error>(())
    /// ```
    pub fn with_base_url(base_url: String) -> Result<Self> {
        validate_base_url(&base_url)?;
        Ok(Self {
            base_url,
            ..Self::default()
        })
    }

    /// Creates a new `ClientConfig` with `base_url` and `timeout`, using the default `max_retries` (3).
    ///
    /// # Errors
    ///
    /// Returns an error if the base URL is invalid or uses an unsupported scheme.
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
    /// )?;
    /// assert_eq!(config.max_retries(), 3);
    /// # Ok::<(), ollama_oxide::Error>(())
    /// ```
    pub fn with_base_url_and_timeout(base_url: String, timeout: Duration) -> Result<Self> {
        validate_base_url(&base_url)?;
        Ok(Self {
            base_url,
            timeout,
            ..Self::default()
        })
    }

    /// Returns the base URL
    #[inline]
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Returns the request timeout duration
    #[inline]
    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    /// Returns the maximum retry attempts
    #[inline]
    pub fn max_retries(&self) -> u32 {
        self.max_retries
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
