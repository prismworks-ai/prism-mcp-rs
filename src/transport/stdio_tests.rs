// ! Tests for STDIO transport that need improvement
// ! This is a placeholder for better test organization

#![cfg(test)]
#![cfg(not(coverage))]

#[cfg(test)]
mod improved_tests {
    use super::super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_stdio_client_invalid_command() {
        // Test with a command that doesn't exist
        let result = StdioClientTransport::new("/nonexistent/command/that/should/not/exist", vec![]).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_stdio_server_default() {
        let transport = StdioServerTransport::default();
        assert!(!transport.is_running());
    }

    #[test]
    fn test_stdio_server_info() {
        let transport = StdioServerTransport::new();
        let info = transport.server_info();
        assert!(info.contains("STDIO server transport"));
    }
}