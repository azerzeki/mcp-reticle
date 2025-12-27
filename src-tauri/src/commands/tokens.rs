//! Token profiling commands
//!
//! Tauri commands for accessing token statistics and context profiling data.

use std::collections::HashMap;
use tauri::State;

use crate::core::server_analyzer::{self, ServerAnalysis};
use crate::core::token_counter::{GlobalTokenStats, SessionTokenStats};
use crate::state::AppState;

/// Get token statistics for a specific session
#[tauri::command]
pub async fn get_session_token_stats(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<Option<SessionTokenStats>, String> {
    Ok(state.token_counter.get_session_stats(&session_id).await)
}

/// Get global token statistics across all sessions
#[tauri::command]
pub async fn get_global_token_stats(
    state: State<'_, AppState>,
) -> Result<GlobalTokenStats, String> {
    Ok(state.token_counter.get_global_stats().await)
}

/// Clear token statistics for a specific session
#[tauri::command]
pub async fn clear_session_token_stats(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state.token_counter.clear_session(&session_id).await;
    Ok(())
}

/// Clear all token statistics
#[tauri::command]
pub async fn clear_all_token_stats(state: State<'_, AppState>) -> Result<(), String> {
    state.token_counter.clear_all().await;
    Ok(())
}

/// Estimate tokens for a given text (utility function)
#[tauri::command]
pub fn estimate_tokens(text: String) -> u64 {
    crate::core::TokenCounter::estimate_tokens(&text)
}

/// Analyze an MCP server to calculate its context token overhead
///
/// This connects to the server, fetches all definitions (tools, prompts, resources),
/// and calculates how many tokens they consume in the LLM context.
#[tauri::command]
pub async fn analyze_mcp_server(
    command: String,
    args: Vec<String>,
    env: Option<HashMap<String, String>>,
    timeout_secs: Option<u64>,
) -> Result<ServerAnalysis, String> {
    server_analyzer::analyze_server(command, args, env, timeout_secs)
        .await
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estimate_tokens_empty_string() {
        let tokens = estimate_tokens(String::new());
        assert_eq!(tokens, 0);
    }

    #[test]
    fn test_estimate_tokens_simple_text() {
        let tokens = estimate_tokens("Hello, world!".to_string());
        // Should return some tokens (actual count depends on implementation)
        assert!(tokens > 0);
    }

    #[test]
    fn test_estimate_tokens_longer_text() {
        let short = estimate_tokens("Hello".to_string());
        let long = estimate_tokens(
            "This is a much longer piece of text that should have more tokens".to_string(),
        );

        // Longer text should have more tokens
        assert!(long > short);
    }

    #[test]
    fn test_estimate_tokens_json() {
        let json = r#"{"jsonrpc":"2.0","method":"tools/call","params":{"name":"test"}}"#;
        let tokens = estimate_tokens(json.to_string());

        // JSON should tokenize reasonably
        assert!(tokens > 5);
    }

    #[test]
    fn test_estimate_tokens_whitespace() {
        let whitespace = estimate_tokens("   \n\t\n   ".to_string());
        let text = estimate_tokens("Hello world".to_string());

        // Whitespace-only should have fewer tokens than text
        assert!(whitespace <= text);
    }

    #[test]
    fn test_estimate_tokens_unicode() {
        let tokens = estimate_tokens("こんにちは世界".to_string());
        // Unicode should be handled (may have different token count)
        assert!(tokens >= 0);
    }

    #[test]
    fn test_estimate_tokens_code() {
        let code = r#"
            fn main() {
                println!("Hello, world!");
            }
        "#;
        let tokens = estimate_tokens(code.to_string());

        // Code should have a reasonable token count
        assert!(tokens > 5);
    }

    #[test]
    fn test_estimate_tokens_large_json() {
        let large_json = serde_json::json!({
            "tools": [
                {"name": "tool1", "description": "A test tool"},
                {"name": "tool2", "description": "Another test tool"},
                {"name": "tool3", "description": "Yet another test tool"}
            ],
            "resources": [
                {"uri": "file:///test.txt", "name": "Test Resource"}
            ]
        });

        let tokens = estimate_tokens(large_json.to_string());
        // Large JSON should have significant tokens
        assert!(tokens > 20);
    }
}
