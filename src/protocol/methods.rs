// ! MCP Protocol Method Constants
// !
// ! Module contains all the method name constants used in the MCP protocol
// ! as defined in the 2025-03-26 specification.

// Core protocol methods
pub const INITIALIZE: &str = "initialize";
pub const INITIALIZED: &str = "notifications/initialized";
pub const PING: &str = "ping";

// Tool-related methods
pub const TOOLS_LIST: &str = "tools/list";
pub const TOOLS_CALL: &str = "tools/call";
pub const TOOLS_LIST_CHANGED: &str = "notifications/tools/list_changed";

// Resource-related methods
pub const RESOURCES_LIST: &str = "resources/list";
pub const RESOURCES_TEMPLATES_LIST: &str = "resources/templates/list"; // New in 2025-06-18
pub const RESOURCES_READ: &str = "resources/read";
pub const RESOURCES_SUBSCRIBE: &str = "resources/subscribe";
pub const RESOURCES_UNSUBSCRIBE: &str = "resources/unsubscribe";
pub const RESOURCES_UPDATED: &str = "notifications/resources/updated";
pub const RESOURCES_LIST_CHANGED: &str = "notifications/resources/list_changed";

// Prompt-related methods
pub const PROMPTS_LIST: &str = "prompts/list";
pub const PROMPTS_GET: &str = "prompts/get";
pub const PROMPTS_LIST_CHANGED: &str = "notifications/prompts/list_changed";

// Sampling methods
pub const SAMPLING_CREATE_MESSAGE: &str = "sampling/createMessage";

// Root-related methods (New in 2025-06-18)
pub const ROOTS_LIST: &str = "roots/list";
pub const ROOTS_LIST_CHANGED: &str = "notifications/roots/list_changed";

// Completion methods (New in 2025-06-18)
pub const COMPLETION_COMPLETE: &str = "completion/complete";

// Elicitation methods (New in 2025-06-18)
pub const ELICITATION_CREATE: &str = "elicitation/create";

// Logging methods
pub const LOGGING_SET_LEVEL: &str = "logging/setLevel";
pub const LOGGING_MESSAGE: &str = "notifications/message";

// Progress and notification methods
pub const PROGRESS: &str = "notifications/progress";
pub const CANCELLED: &str = "notifications/cancelled"; // New in 2025-06-18

// Discovery methods (Optional RPC discovery mechanism)
pub const RPC_DISCOVER: &str = "rpc.discover";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_method_constants_valid() {
        // Test that all method constants are valid strings
        let methods = vec![
            INITIALIZE,
            INITIALIZED,
            PING,
            TOOLS_LIST,
            TOOLS_CALL,
            TOOLS_LIST_CHANGED,
            RESOURCES_LIST,
            RESOURCES_TEMPLATES_LIST,
            RESOURCES_READ,
            RESOURCES_SUBSCRIBE,
            RESOURCES_UNSUBSCRIBE,
            RESOURCES_UPDATED,
            RESOURCES_LIST_CHANGED,
            PROMPTS_LIST,
            PROMPTS_GET,
            PROMPTS_LIST_CHANGED,
            SAMPLING_CREATE_MESSAGE,
            ROOTS_LIST,
            ROOTS_LIST_CHANGED,
            COMPLETION_COMPLETE,
            ELICITATION_CREATE,
            LOGGING_SET_LEVEL,
            LOGGING_MESSAGE,
            PROGRESS,
            CANCELLED,
        ];

        for method in methods {
            assert!(
                !method.is_empty(),
                "Method constant should not be empty: {method}"
            );
            assert!(
                !method.contains(' '),
                "Method constant should not contain spaces: {method}"
            );
        }
    }

    #[test]
    fn test_method_name_consistency() {
        // Test core protocol methods
        assert_eq!(INITIALIZE, "initialize");
        assert_eq!(INITIALIZED, "notifications/initialized");
        assert_eq!(PING, "ping");

        // Test tool methods
        assert_eq!(TOOLS_LIST, "tools/list");
        assert_eq!(TOOLS_CALL, "tools/call");
        assert_eq!(TOOLS_LIST_CHANGED, "notifications/tools/list_changed");

        // Test resource methods
        assert_eq!(RESOURCES_LIST, "resources/list");
        assert_eq!(RESOURCES_TEMPLATES_LIST, "resources/templates/list");
        assert_eq!(RESOURCES_READ, "resources/read");
        assert_eq!(RESOURCES_SUBSCRIBE, "resources/subscribe");
        assert_eq!(RESOURCES_UNSUBSCRIBE, "resources/unsubscribe");
        assert_eq!(RESOURCES_UPDATED, "notifications/resources/updated");
        assert_eq!(
            RESOURCES_LIST_CHANGED,
            "notifications/resources/list_changed"
        );

        // Test prompt methods
        assert_eq!(PROMPTS_LIST, "prompts/list");
        assert_eq!(PROMPTS_GET, "prompts/get");
        assert_eq!(PROMPTS_LIST_CHANGED, "notifications/prompts/list_changed");

        // Test sampling methods
        assert_eq!(SAMPLING_CREATE_MESSAGE, "sampling/createMessage");

        // Test 2025-06-18 new methods
        assert_eq!(ROOTS_LIST, "roots/list");
        assert_eq!(ROOTS_LIST_CHANGED, "notifications/roots/list_changed");
        assert_eq!(COMPLETION_COMPLETE, "completion/complete");
        assert_eq!(ELICITATION_CREATE, "elicitation/create");
        assert_eq!(CANCELLED, "notifications/cancelled");

        // Test logging methods
        assert_eq!(LOGGING_SET_LEVEL, "logging/setLevel");
        assert_eq!(LOGGING_MESSAGE, "notifications/message");
        assert_eq!(PROGRESS, "notifications/progress");
    }

    #[test]
    fn test_notification_methods_prefix() {
        // All notification methods should start with "notifications/"
        let notification_methods = vec![
            INITIALIZED,
            TOOLS_LIST_CHANGED,
            RESOURCES_UPDATED,
            RESOURCES_LIST_CHANGED,
            PROMPTS_LIST_CHANGED,
            ROOTS_LIST_CHANGED,
            LOGGING_MESSAGE,
            PROGRESS,
            CANCELLED,
        ];

        for method in notification_methods {
            assert!(
                method.starts_with("notifications/"),
                "Notification method should start with 'notifications/': {method}"
            );
        }
    }

    #[test]
    fn test_request_methods_no_notification_prefix() {
        // Request methods should not start with "notifications/"
        let request_methods = vec![
            INITIALIZE,
            PING,
            TOOLS_LIST,
            TOOLS_CALL,
            RESOURCES_LIST,
            RESOURCES_TEMPLATES_LIST,
            RESOURCES_READ,
            RESOURCES_SUBSCRIBE,
            RESOURCES_UNSUBSCRIBE,
            PROMPTS_LIST,
            PROMPTS_GET,
            SAMPLING_CREATE_MESSAGE,
            ROOTS_LIST,
            COMPLETION_COMPLETE,
            ELICITATION_CREATE,
            LOGGING_SET_LEVEL,
        ];

        for method in request_methods {
            assert!(
                !method.starts_with("notifications/"),
                "Request method should not start with 'notifications/': {method}"
            );
        }
    }

    #[test]
    fn test_method_categories() {
        // Test that methods are properly categorized
        let tool_methods = vec![TOOLS_LIST, TOOLS_CALL, TOOLS_LIST_CHANGED];
        for method in tool_methods {
            assert!(
                method.contains("tools"),
                "Tool method should contain 'tools': {method}"
            );
        }

        let resource_methods = vec![
            RESOURCES_LIST,
            RESOURCES_TEMPLATES_LIST,
            RESOURCES_READ,
            RESOURCES_SUBSCRIBE,
            RESOURCES_UNSUBSCRIBE,
            RESOURCES_UPDATED,
            RESOURCES_LIST_CHANGED,
        ];
        for method in resource_methods {
            assert!(
                method.contains("resources"),
                "Resource method should contain 'resources': {method}"
            );
        }

        let prompt_methods = vec![PROMPTS_LIST, PROMPTS_GET, PROMPTS_LIST_CHANGED];
        for method in prompt_methods {
            assert!(
                method.contains("prompts"),
                "Prompt method should contain 'prompts': {method}"
            );
        }
    }

    #[test]
    fn test_2025_06_18_new_methods() {
        // Test that new methods introduced in 2025-06-18 are present
        let new_methods = vec![
            RESOURCES_TEMPLATES_LIST,
            ROOTS_LIST,
            ROOTS_LIST_CHANGED,
            COMPLETION_COMPLETE,
            ELICITATION_CREATE,
            CANCELLED,
        ];

        // Just verify they exist and are not empty
        for method in new_methods {
            assert!(
                !method.is_empty(),
                "New 2025-06-18 method should not be empty: {method}"
            );
        }
    }

    #[test]
    fn test_method_constants_unique() {
        // Test that all method constants are unique
        let methods = vec![
            INITIALIZE,
            INITIALIZED,
            PING,
            TOOLS_LIST,
            TOOLS_CALL,
            TOOLS_LIST_CHANGED,
            RESOURCES_LIST,
            RESOURCES_TEMPLATES_LIST,
            RESOURCES_READ,
            RESOURCES_SUBSCRIBE,
            RESOURCES_UNSUBSCRIBE,
            RESOURCES_UPDATED,
            RESOURCES_LIST_CHANGED,
            PROMPTS_LIST,
            PROMPTS_GET,
            PROMPTS_LIST_CHANGED,
            SAMPLING_CREATE_MESSAGE,
            ROOTS_LIST,
            ROOTS_LIST_CHANGED,
            COMPLETION_COMPLETE,
            ELICITATION_CREATE,
            LOGGING_SET_LEVEL,
            LOGGING_MESSAGE,
            PROGRESS,
            CANCELLED,
        ];

        let mut unique_methods = std::collections::HashSet::new();
        for method in methods {
            assert!(
                unique_methods.insert(method),
                "Duplicate method constant found: {method}"
            );
        }
    }

    #[test]
    fn test_method_naming_conventions() {
        // Test that method names follow proper conventions
        // Should use lowercase with underscores for separators in constant names
        // Should use camelCase in the actual method strings where appropriate

        // Check camelCase in method strings
        assert!(SAMPLING_CREATE_MESSAGE.contains("createMessage"));
        assert!(LOGGING_SET_LEVEL.contains("setLevel"));

        // Check slash separators for namespacing
        assert!(TOOLS_LIST.contains("/"));
        assert!(RESOURCES_LIST.contains("/"));
        assert!(PROMPTS_LIST.contains("/"));
        assert!(COMPLETION_COMPLETE.contains("/"));
    }
}
