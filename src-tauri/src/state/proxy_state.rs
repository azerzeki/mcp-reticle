use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::process::ChildStdin;
use tokio::sync::Mutex;

/// Wrapper for child stdin that can be shared across threads
pub type SharedChildStdin = Arc<Mutex<Option<ChildStdin>>>;

/// Runtime state for the proxy process
///
/// This tracks whether the demo/proxy is currently running
/// and which session it's associated with.
#[derive(Debug, Default)]
pub struct ProxyState {
    /// Current session ID if proxy is running
    pub session_id: Option<String>,

    /// Whether the proxy is currently running
    pub is_running: bool,

    /// Child process stdin handle for sending messages (stdio transport only)
    pub child_stdin: Option<SharedChildStdin>,

    /// HTTP proxy URL for sending messages (HTTP/SSE transport only)
    /// Format: "http://localhost:3001" (the proxy port)
    pub http_proxy_url: Option<String>,
}

impl ProxyState {
    /// Create a new proxy state (not running)
    pub fn new() -> Self {
        Self::default()
    }

    /// Start the proxy with a session ID
    pub fn start(&mut self, session_id: String) {
        self.session_id = Some(session_id);
        self.is_running = true;
    }

    /// Start the proxy with stdin handle for interaction support (stdio transport)
    pub fn start_with_stdin(&mut self, session_id: String, stdin: ChildStdin) {
        self.session_id = Some(session_id);
        self.is_running = true;
        self.child_stdin = Some(Arc::new(Mutex::new(Some(stdin))));
        self.http_proxy_url = None;
    }

    /// Start the proxy with HTTP URL for interaction support (HTTP/SSE transport)
    pub fn start_with_http(&mut self, session_id: String, proxy_url: String) {
        self.session_id = Some(session_id);
        self.is_running = true;
        self.child_stdin = None;
        self.http_proxy_url = Some(proxy_url);
    }

    /// Stop the proxy
    pub fn stop(&mut self) {
        self.session_id = None;
        self.is_running = false;
        self.child_stdin = None;
        self.http_proxy_url = None;
    }

    /// Check if proxy is running
    pub fn is_running(&self) -> bool {
        self.is_running
    }

    /// Get current session ID if running
    #[allow(dead_code)]
    pub fn session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }

    /// Check if interaction is available (either stdio or HTTP)
    pub fn can_send(&self) -> bool {
        self.is_running && (self.child_stdin.is_some() || self.http_proxy_url.is_some())
    }

    /// Check if using stdio transport
    pub fn is_stdio(&self) -> bool {
        self.child_stdin.is_some()
    }

    /// Check if using HTTP transport
    pub fn is_http(&self) -> bool {
        self.http_proxy_url.is_some()
    }

    /// Get HTTP proxy URL if available
    pub fn get_http_proxy_url(&self) -> Option<&str> {
        self.http_proxy_url.as_deref()
    }

    /// Send a message to the child process via stdin
    pub async fn send_message(&self, message: &str) -> Result<(), String> {
        let stdin_arc = self.child_stdin.as_ref().ok_or_else(|| {
            "No stdin available - proxy not running or using HTTP transport".to_string()
        })?;

        let mut stdin_guard = stdin_arc.lock().await;
        let stdin = stdin_guard
            .as_mut()
            .ok_or_else(|| "Stdin has been closed".to_string())?;

        // Write message followed by newline (JSON-RPC over stdio uses \n as delimiter)
        let msg_with_newline = format!("{message}\n");
        stdin
            .write_all(msg_with_newline.as_bytes())
            .await
            .map_err(|e| format!("Failed to write to stdin: {e}"))?;

        stdin
            .flush()
            .await
            .map_err(|e| format!("Failed to flush stdin: {e}"))?;

        Ok(())
    }
}
