//! Tests for chat API methods (POST /api/chat)

use ollama_oxide::{
    ChatMessage, ChatRequest, ChatResponse, ChatRole, ClientConfig, FormatSetting,
    KeepAliveSetting, ModelOptions, OllamaApiAsync, OllamaApiSync, OllamaClient, ResponseMessage,
    ThinkSetting,
};
#[cfg(feature = "tools")]
use ollama_oxide::{ToolCall, ToolCallFunction, ToolDefinition, ToolFunction};
use serde_json::json;
use std::time::Duration;

// ============================================================================
// ChatRole Type Tests
// ============================================================================

#[test]
fn test_chat_role_serialization() {
    assert_eq!(
        serde_json::to_string(&ChatRole::System).unwrap(),
        "\"system\""
    );
    assert_eq!(serde_json::to_string(&ChatRole::User).unwrap(), "\"user\"");
    assert_eq!(
        serde_json::to_string(&ChatRole::Assistant).unwrap(),
        "\"assistant\""
    );
    assert_eq!(serde_json::to_string(&ChatRole::Tool).unwrap(), "\"tool\"");
}

#[test]
fn test_chat_role_deserialization() {
    assert_eq!(
        serde_json::from_str::<ChatRole>("\"system\"").unwrap(),
        ChatRole::System
    );
    assert_eq!(
        serde_json::from_str::<ChatRole>("\"user\"").unwrap(),
        ChatRole::User
    );
    assert_eq!(
        serde_json::from_str::<ChatRole>("\"assistant\"").unwrap(),
        ChatRole::Assistant
    );
    assert_eq!(
        serde_json::from_str::<ChatRole>("\"tool\"").unwrap(),
        ChatRole::Tool
    );
}

#[test]
fn test_chat_role_default() {
    assert_eq!(ChatRole::default(), ChatRole::User);
}

#[test]
fn test_chat_role_display() {
    assert_eq!(ChatRole::System.to_string(), "system");
    assert_eq!(ChatRole::User.to_string(), "user");
    assert_eq!(ChatRole::Assistant.to_string(), "assistant");
    assert_eq!(ChatRole::Tool.to_string(), "tool");
}

// ============================================================================
// ChatMessage Type Tests
// ============================================================================

#[test]
fn test_chat_message_system() {
    let msg = ChatMessage::system("You are a helpful assistant.");
    assert_eq!(msg.role, ChatRole::System);
    assert_eq!(msg.content, "You are a helpful assistant.");
    assert!(msg.is_system());
}

#[test]
fn test_chat_message_user() {
    let msg = ChatMessage::user("Hello!");
    assert_eq!(msg.role, ChatRole::User);
    assert_eq!(msg.content, "Hello!");
    assert!(msg.is_user());
}

#[test]
fn test_chat_message_assistant() {
    let msg = ChatMessage::assistant("Hi there!");
    assert_eq!(msg.role, ChatRole::Assistant);
    assert_eq!(msg.content, "Hi there!");
    assert!(msg.is_assistant());
}

#[test]
fn test_chat_message_tool() {
    let msg = ChatMessage::tool(r#"{"result": 42}"#);
    assert_eq!(msg.role, ChatRole::Tool);
    assert!(msg.is_tool());
}

#[test]
fn test_chat_message_with_image() {
    let msg = ChatMessage::user("Describe this image.").with_image("base64data");
    assert!(msg.has_images());
    assert_eq!(msg.images.as_ref().unwrap().len(), 1);
}

#[test]
fn test_chat_message_with_multiple_images() {
    let msg = ChatMessage::user("Compare").with_images(vec!["img1", "img2"]);
    assert_eq!(msg.images.as_ref().unwrap().len(), 2);
}

#[cfg(feature = "tools")]
#[test]
fn test_chat_message_with_tool_calls() {
    let call = ToolCall::new(ToolCallFunction::new("test"));
    let msg = ChatMessage::assistant("").with_tool_calls(vec![call]);
    assert!(msg.has_tool_calls());
}

#[test]
fn test_chat_message_serialization() {
    let msg = ChatMessage::user("Hello");
    let json = serde_json::to_value(&msg).unwrap();

    assert_eq!(json["role"], "user");
    assert_eq!(json["content"], "Hello");
    assert!(json.get("images").is_none()); // Skipped when None
}

#[test]
fn test_chat_message_deserialization() {
    let json = r#"{"role": "user", "content": "Hello"}"#;
    let msg: ChatMessage = serde_json::from_str(json).unwrap();

    assert_eq!(msg.role, ChatRole::User);
    assert_eq!(msg.content, "Hello");
}

// ============================================================================
// ToolFunction Type Tests (requires "tools" feature)
// ============================================================================

#[cfg(feature = "tools")]
#[test]
fn test_tool_function_new() {
    let func = ToolFunction::new("get_time", json!({"type": "object", "properties": {}}));
    assert_eq!(func.name, "get_time");
    assert!(func.description.is_none());
}

#[cfg(feature = "tools")]
#[test]
fn test_tool_function_with_description() {
    let func = ToolFunction::new("search", json!({})).with_description("Search the web");
    assert_eq!(func.description, Some("Search the web".to_string()));
}

#[cfg(feature = "tools")]
#[test]
fn test_tool_function_no_params() {
    let func = ToolFunction::no_params("get_time");
    assert_eq!(func.parameters["type"], "object");
}

#[cfg(feature = "tools")]
#[test]
fn test_tool_function_serialization() {
    let func = ToolFunction::new(
        "get_weather",
        json!({
            "type": "object",
            "properties": {
                "location": {"type": "string"}
            }
        }),
    )
    .with_description("Get weather");

    let json = serde_json::to_value(&func).unwrap();
    assert_eq!(json["name"], "get_weather");
    assert_eq!(json["description"], "Get weather");
}

// ============================================================================
// ToolDefinition Type Tests (requires "tools" feature)
// ============================================================================

#[cfg(feature = "tools")]
#[test]
fn test_tool_definition_function() {
    let tool = ToolDefinition::function("test", json!({"type": "object"}));
    assert_eq!(tool.type_field, "function");
    assert_eq!(tool.function.name, "test");
}

#[cfg(feature = "tools")]
#[test]
fn test_tool_definition_function_no_params() {
    let tool = ToolDefinition::function_no_params("get_time");
    assert_eq!(tool.name(), "get_time");
}

#[cfg(feature = "tools")]
#[test]
fn test_tool_definition_with_description() {
    let tool = ToolDefinition::function("test", json!({})).with_description("A test tool");
    assert_eq!(tool.description(), Some("A test tool"));
}

#[cfg(feature = "tools")]
#[test]
fn test_tool_definition_serialization() {
    let tool = ToolDefinition::function(
        "get_weather",
        json!({
            "type": "object",
            "properties": {"location": {"type": "string"}}
        }),
    )
    .with_description("Get weather");

    let json = serde_json::to_value(&tool).unwrap();
    assert_eq!(json["type"], "function");
    assert_eq!(json["function"]["name"], "get_weather");
}

// ============================================================================
// ToolCallFunction Type Tests (requires "tools" feature)
// ============================================================================

#[cfg(feature = "tools")]
#[test]
fn test_tool_call_function_new() {
    let func = ToolCallFunction::new("test");
    assert_eq!(func.name, "test");
    assert!(func.arguments.is_none());
}

#[cfg(feature = "tools")]
#[test]
fn test_tool_call_function_with_arguments() {
    let func = ToolCallFunction::with_arguments("calc", json!({"x": 42}));
    assert_eq!(func.name, "calc");
    assert_eq!(func.arguments.as_ref().unwrap()["x"], 42);
}

#[cfg(feature = "tools")]
#[test]
fn test_tool_call_function_arguments_as() {
    #[derive(serde::Deserialize, PartialEq, Debug)]
    struct Args {
        x: i32,
    }

    let func = ToolCallFunction::with_arguments("test", json!({"x": 42}));
    let args: Option<Args> = func.arguments_as();
    assert_eq!(args.unwrap().x, 42);
}

#[cfg(feature = "tools")]
#[test]
fn test_tool_call_function_deserialization() {
    let json = r#"{"name": "get_weather", "arguments": {"location": "Paris"}}"#;
    let func: ToolCallFunction = serde_json::from_str(json).unwrap();

    assert_eq!(func.name, "get_weather");
    assert_eq!(func.arguments.unwrap()["location"], "Paris");
}

// ============================================================================
// ToolCall Type Tests (requires "tools" feature)
// ============================================================================

#[cfg(feature = "tools")]
#[test]
fn test_tool_call_new() {
    let call = ToolCall::new(ToolCallFunction::new("test"));
    assert!(call.is_valid());
    assert_eq!(call.function_name(), Some("test"));
}

#[cfg(feature = "tools")]
#[test]
fn test_tool_call_default() {
    let call = ToolCall::default();
    assert!(!call.is_valid());
    assert!(call.function_name().is_none());
}

#[cfg(feature = "tools")]
#[test]
fn test_tool_call_arguments() {
    let call = ToolCall::new(ToolCallFunction::with_arguments("calc", json!({"x": 1})));
    assert_eq!(call.arguments().unwrap()["x"], 1);
}

#[cfg(feature = "tools")]
#[test]
fn test_tool_call_deserialization() {
    let json = r#"{"function": {"name": "test", "arguments": {"a": 1}}}"#;
    let call: ToolCall = serde_json::from_str(json).unwrap();

    assert_eq!(call.function_name(), Some("test"));
    assert_eq!(call.arguments().unwrap()["a"], 1);
}

// ============================================================================
// ResponseMessage Type Tests
// ============================================================================

#[test]
fn test_response_message_new() {
    let msg = ResponseMessage::new("Hello!");
    assert_eq!(msg.content(), Some("Hello!"));
    assert!(msg.has_content());
}

#[test]
fn test_response_message_empty() {
    let msg = ResponseMessage::empty();
    assert!(!msg.has_content());
}

#[cfg(feature = "tools")]
#[test]
fn test_response_message_with_tool_calls() {
    let json = r#"{
        "role": "assistant",
        "content": "",
        "tool_calls": [{"function": {"name": "test"}}]
    }"#;

    let msg: ResponseMessage = serde_json::from_str(json).unwrap();
    assert!(msg.has_tool_calls());
}

#[test]
fn test_response_message_thinking() {
    let json = r#"{
        "role": "assistant",
        "content": "42",
        "thinking": "Let me think..."
    }"#;

    let msg: ResponseMessage = serde_json::from_str(json).unwrap();
    assert_eq!(msg.thinking(), Some("Let me think..."));
    assert!(msg.has_thinking());
}

// ============================================================================
// ChatRequest Type Tests
// ============================================================================

#[test]
fn test_chat_request_new() {
    let request = ChatRequest::new("qwen3:0.6b", [ChatMessage::user("Hello")]);

    assert_eq!(request.model, "qwen3:0.6b");
    assert_eq!(request.messages.len(), 1);
    assert_eq!(request.stream, Some(false)); // Non-streaming for v0.1.0
}

#[test]
fn test_chat_request_new_with_iterator() {
    let msgs = ["Hi", "Bye"];
    let request = ChatRequest::new("model", msgs.iter().map(|s| ChatMessage::user(*s)));
    assert_eq!(request.messages.len(), 2);
}

#[test]
fn test_chat_request_with_message() {
    let request = ChatRequest::new("model", [ChatMessage::user("1")])
        .with_message(ChatMessage::assistant("2"));
    assert_eq!(request.messages.len(), 2);
}

#[cfg(feature = "tools")]
#[test]
fn test_chat_request_with_tools() {
    let tool = ToolDefinition::function("test", json!({}));
    let request = ChatRequest::new("model", [ChatMessage::user("Hi")]).with_tools(vec![tool]);

    assert!(request.has_tools());
    assert_eq!(request.tools().unwrap().len(), 1);
}

#[cfg(feature = "tools")]
#[test]
fn test_chat_request_with_tool() {
    let request = ChatRequest::new("model", [ChatMessage::user("Hi")])
        .with_tool(ToolDefinition::function("a", json!({})))
        .with_tool(ToolDefinition::function("b", json!({})));

    assert_eq!(request.tools().unwrap().len(), 2);
}

#[test]
fn test_chat_request_with_format() {
    let request =
        ChatRequest::new("model", [ChatMessage::user("Hi")]).with_format(FormatSetting::json());
    assert!(request.format.is_some());
}

#[test]
fn test_chat_request_with_options() {
    let options = ModelOptions::default().with_temperature(0.7);
    let request = ChatRequest::new("model", [ChatMessage::user("Hi")]).with_options(options);

    assert_eq!(request.options.unwrap().temperature, Some(0.7));
}

#[test]
fn test_chat_request_with_think() {
    let request =
        ChatRequest::new("model", [ChatMessage::user("Hi")]).with_think(ThinkSetting::enabled());
    assert!(request.think.is_some());
}

#[test]
fn test_chat_request_with_keep_alive() {
    let request = ChatRequest::new("model", [ChatMessage::user("Hi")])
        .with_keep_alive(KeepAliveSetting::duration("5m"));
    assert!(request.keep_alive.is_some());
}

#[test]
fn test_chat_request_with_logprobs() {
    let request = ChatRequest::new("model", [ChatMessage::user("Hi")])
        .with_logprobs(true)
        .with_top_logprobs(5);

    assert_eq!(request.logprobs, Some(true));
    assert_eq!(request.top_logprobs, Some(5));
}

#[test]
fn test_chat_request_serialization_minimal() {
    let request = ChatRequest::new("qwen3:0.6b", [ChatMessage::user("Hello")]);
    let json = serde_json::to_string(&request).unwrap();

    assert!(json.contains(r#""model":"qwen3:0.6b""#));
    assert!(json.contains(r#""stream":false"#));
    assert!(json.contains(r#""role":"user""#));
    assert!(json.contains(r#""content":"Hello""#));
}

#[cfg(feature = "tools")]
#[test]
fn test_chat_request_serialization_with_tools() {
    let request = ChatRequest::new("model", [ChatMessage::user("Hi")]).with_tools(vec![
        ToolDefinition::function(
            "get_weather",
            json!({"type": "object", "properties": {"location": {"type": "string"}}}),
        )
        .with_description("Get weather"),
    ]);

    let json = serde_json::to_value(&request).unwrap();
    assert_eq!(json["tools"][0]["type"], "function");
    assert_eq!(json["tools"][0]["function"]["name"], "get_weather");
}

// ============================================================================
// ChatResponse Type Tests
// ============================================================================

#[test]
fn test_chat_response_default() {
    let response = ChatResponse::default();
    assert!(!response.is_done());
    assert!(response.content().is_none());
}

#[test]
fn test_chat_response_content() {
    let json = r#"{"message": {"role": "assistant", "content": "Hello!"}}"#;
    let response: ChatResponse = serde_json::from_str(json).unwrap();
    assert_eq!(response.content(), Some("Hello!"));
}

#[cfg(feature = "tools")]
#[test]
fn test_chat_response_tool_calls() {
    let json = r#"{
        "message": {
            "role": "assistant",
            "content": "",
            "tool_calls": [{"function": {"name": "get_weather", "arguments": {"location": "Paris"}}}]
        },
        "done": true
    }"#;

    let response: ChatResponse = serde_json::from_str(json).unwrap();
    assert!(response.has_tool_calls());
    let calls = response.tool_calls().unwrap();
    assert_eq!(calls[0].function_name(), Some("get_weather"));
}

#[test]
fn test_chat_response_is_done() {
    let json = r#"{"done": true}"#;
    let response: ChatResponse = serde_json::from_str(json).unwrap();
    assert!(response.is_done());
}

#[test]
fn test_chat_response_duration_conversions() {
    let json = r#"{
        "total_duration": 1000000000,
        "load_duration": 500000000,
        "prompt_eval_duration": 200000000,
        "eval_duration": 300000000
    }"#;
    let response: ChatResponse = serde_json::from_str(json).unwrap();

    assert_eq!(response.total_duration_ms(), Some(1000.0));
    assert_eq!(response.load_duration_ms(), Some(500.0));
    assert_eq!(response.prompt_eval_duration_ms(), Some(200.0));
    assert_eq!(response.eval_duration_ms(), Some(300.0));
}

#[test]
fn test_chat_response_tokens_per_second() {
    let json = r#"{"eval_count": 100, "eval_duration": 2000000000}"#;
    let response: ChatResponse = serde_json::from_str(json).unwrap();
    assert_eq!(response.tokens_per_second(), Some(50.0));
}

#[test]
fn test_chat_response_token_counts() {
    let json = r#"{"prompt_eval_count": 10, "eval_count": 50}"#;
    let response: ChatResponse = serde_json::from_str(json).unwrap();

    assert_eq!(response.prompt_tokens(), Some(10));
    assert_eq!(response.completion_tokens(), Some(50));
    assert_eq!(response.total_tokens(), Some(60));
}

#[test]
fn test_chat_response_deserialization_full() {
    let json = r#"{
        "model": "qwen3:0.6b",
        "created_at": "2025-10-17T23:14:07.414671Z",
        "message": {
            "role": "assistant",
            "content": "Hello! How can I help you today?"
        },
        "done": true,
        "done_reason": "stop",
        "total_duration": 174560334,
        "load_duration": 101397084,
        "prompt_eval_count": 11,
        "prompt_eval_duration": 13074791,
        "eval_count": 18,
        "eval_duration": 52479709
    }"#;

    let response: ChatResponse = serde_json::from_str(json).unwrap();

    assert_eq!(response.model(), Some("qwen3:0.6b"));
    assert_eq!(response.content(), Some("Hello! How can I help you today?"));
    assert!(response.is_done());
    assert_eq!(response.done_reason(), Some("stop"));
    assert_eq!(response.prompt_tokens(), Some(11));
    assert_eq!(response.completion_tokens(), Some(18));
}

// ============================================================================
// Async API Tests
// ============================================================================

#[tokio::test]
async fn test_chat_async_success() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("POST", "/api/chat")
        .match_body(mockito::Matcher::Json(json!({
            "model": "qwen3:0.6b",
            "messages": [{"role": "user", "content": "Hello"}],
            "stream": false
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "model": "qwen3:0.6b",
            "message": {"role": "assistant", "content": "Hello! How can I help?"},
            "done": true,
            "done_reason": "stop"
        }"#,
        )
        .create_async()
        .await;

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(5),
        0,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let request = ChatRequest::new("qwen3:0.6b", [ChatMessage::user("Hello")]);
    let response = client.chat(&request).await.unwrap();

    assert_eq!(response.model(), Some("qwen3:0.6b"));
    assert_eq!(response.content(), Some("Hello! How can I help?"));
    assert!(response.is_done());

    mock.assert_async().await;
}

#[tokio::test]
async fn test_chat_async_multi_turn() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("POST", "/api/chat")
        .match_body(mockito::Matcher::Json(json!({
            "model": "qwen3:0.6b",
            "messages": [
                {"role": "system", "content": "Be helpful"},
                {"role": "user", "content": "What is Rust?"},
                {"role": "assistant", "content": "Rust is a programming language."},
                {"role": "user", "content": "Why use it?"}
            ],
            "stream": false
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"message": {"content": "Memory safety!"}, "done": true}"#)
        .create_async()
        .await;

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(5),
        0,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let request = ChatRequest::new(
        "qwen3:0.6b",
        [
            ChatMessage::system("Be helpful"),
            ChatMessage::user("What is Rust?"),
            ChatMessage::assistant("Rust is a programming language."),
            ChatMessage::user("Why use it?"),
        ],
    );
    let response = client.chat(&request).await.unwrap();

    assert_eq!(response.content(), Some("Memory safety!"));

    mock.assert_async().await;
}

#[cfg(feature = "tools")]
#[tokio::test]
async fn test_chat_async_with_tools() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("POST", "/api/chat")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "message": {
                "role": "assistant",
                "content": "",
                "tool_calls": [
                    {"function": {"name": "get_weather", "arguments": {"location": "Paris"}}}
                ]
            },
            "done": true
        }"#,
        )
        .create_async()
        .await;

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(5),
        0,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let request = ChatRequest::new("model", [ChatMessage::user("What's the weather in Paris?")])
        .with_tools(vec![ToolDefinition::function(
            "get_weather",
            json!({"type": "object", "properties": {"location": {"type": "string"}}}),
        )]);

    let response = client.chat(&request).await.unwrap();

    assert!(response.has_tool_calls());
    let calls = response.tool_calls().unwrap();
    assert_eq!(calls[0].function_name(), Some("get_weather"));
    assert_eq!(calls[0].arguments().unwrap()["location"], "Paris");

    mock.assert_async().await;
}

#[tokio::test]
async fn test_chat_async_model_not_found() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("POST", "/api/chat")
        .with_status(404)
        .create_async()
        .await;

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(5),
        0,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let request = ChatRequest::new("nonexistent", [ChatMessage::user("Hello")]);
    let result = client.chat(&request).await;

    assert!(result.is_err());

    mock.assert_async().await;
}

#[tokio::test]
async fn test_chat_async_retry_on_server_error() {
    let mut server = mockito::Server::new_async().await;

    let mock_fail = server
        .mock("POST", "/api/chat")
        .with_status(500)
        .expect(1)
        .create_async()
        .await;

    let mock_success = server
        .mock("POST", "/api/chat")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"message": {"content": "Ok"}, "done": true}"#)
        .expect(1)
        .create_async()
        .await;

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(5),
        1,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let request = ChatRequest::new("model", [ChatMessage::user("Hello")]);
    let result = client.chat(&request).await;

    assert!(result.is_ok());

    mock_fail.assert_async().await;
    mock_success.assert_async().await;
}

// ============================================================================
// Sync API Tests
// ============================================================================

#[test]
fn test_chat_sync_success() {
    let mut server = mockito::Server::new();

    let mock = server
        .mock("POST", "/api/chat")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "model": "qwen3:0.6b",
            "message": {"role": "assistant", "content": "Hello there!"},
            "done": true
        }"#,
        )
        .create();

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(5),
        0,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let request = ChatRequest::new("qwen3:0.6b", [ChatMessage::user("Hello")]);
    let response = client.chat_blocking(&request).unwrap();

    assert_eq!(response.content(), Some("Hello there!"));

    mock.assert();
}

#[test]
fn test_chat_sync_model_not_found() {
    let mut server = mockito::Server::new();

    let mock = server.mock("POST", "/api/chat").with_status(404).create();

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(5),
        0,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let request = ChatRequest::new("nonexistent", [ChatMessage::user("Hello")]);
    let result = client.chat_blocking(&request);

    assert!(result.is_err());

    mock.assert();
}

#[test]
fn test_chat_sync_retry_on_server_error() {
    let mut server = mockito::Server::new();

    let mock_fail = server
        .mock("POST", "/api/chat")
        .with_status(500)
        .expect(1)
        .create();

    let mock_success = server
        .mock("POST", "/api/chat")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"message": {"content": "Ok"}, "done": true}"#)
        .expect(1)
        .create();

    let config = ClientConfig::new(
        server.url(),
        Duration::from_secs(5),
        1,
    ).unwrap();

    let client = OllamaClient::new(config).unwrap();
    let request = ChatRequest::new("model", [ChatMessage::user("Hello")]);
    let result = client.chat_blocking(&request);

    assert!(result.is_ok());

    mock_fail.assert();
    mock_success.assert();
}

// ============================================================================
// Type Safety Tests
// ============================================================================

#[test]
fn test_chat_role_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<ChatRole>();
}

#[test]
fn test_chat_message_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<ChatMessage>();
}

#[test]
fn test_chat_request_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<ChatRequest>();
}

#[test]
fn test_chat_response_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<ChatResponse>();
}

#[cfg(feature = "tools")]
#[test]
fn test_tool_function_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<ToolFunction>();
}

#[cfg(feature = "tools")]
#[test]
fn test_tool_definition_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<ToolDefinition>();
}

#[cfg(feature = "tools")]
#[test]
fn test_tool_call_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<ToolCall>();
}

#[cfg(feature = "tools")]
#[test]
fn test_tool_call_function_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<ToolCallFunction>();
}

#[test]
fn test_response_message_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<ResponseMessage>();
}

// ============================================================================
// Clone and Debug Tests
// ============================================================================

#[test]
fn test_chat_message_clone() {
    let msg = ChatMessage::user("Hello").with_image("data");
    let cloned = msg.clone();
    assert_eq!(msg, cloned);
}

#[test]
fn test_chat_request_clone() {
    let request =
        ChatRequest::new("model", [ChatMessage::user("Hi")]).with_format(FormatSetting::json());
    let cloned = request.clone();
    assert_eq!(request, cloned);
}

#[test]
fn test_chat_response_clone() {
    let json = r#"{"message": {"content": "Hi"}, "done": true}"#;
    let response: ChatResponse = serde_json::from_str(json).unwrap();
    let cloned = response.clone();
    assert_eq!(response, cloned);
}

#[cfg(feature = "tools")]
#[test]
fn test_tool_definition_clone() {
    let tool = ToolDefinition::function("test", json!({})).with_description("A test");
    let cloned = tool.clone();
    assert_eq!(tool, cloned);
}

#[test]
fn test_chat_message_debug() {
    let msg = ChatMessage::user("Hello");
    let debug_str = format!("{:?}", msg);
    assert!(debug_str.contains("Hello"));
    assert!(debug_str.contains("User"));
}

#[test]
fn test_chat_request_debug() {
    let request = ChatRequest::new("model", [ChatMessage::user("Hi")]);
    let debug_str = format!("{:?}", request);
    assert!(debug_str.contains("model"));
}
