//! Chat request type for POST /api/chat endpoint.

use serde::{Deserialize, Serialize};

use super::{ChatMessage, FormatSetting, KeepAliveSetting, ModelOptions, ThinkSetting};
#[cfg(feature = "tools")]
use crate::tools::ToolDefinition;

/// Request body for POST /api/chat endpoint.
///
/// Generates the next message in a chat conversation.
/// This type always sets `stream: false` for non-streaming responses.
///
/// # Examples
///
/// ## Basic Request
///
/// ```no_run
/// use ollama_oxide::{ChatRequest, ChatMessage};
///
/// let request = ChatRequest::new("qwen3:0.6b", [
///     ChatMessage::user("Hello!")
/// ]);
/// ```
///
/// ## With System Message
///
/// ```no_run
/// use ollama_oxide::{ChatRequest, ChatMessage};
///
/// let request = ChatRequest::new("qwen3:0.6b", [
///     ChatMessage::system("You are a helpful assistant."),
///     ChatMessage::user("What is Rust?")
/// ]);
/// ```
///
/// ## With Tools (Function Calling)
///
/// Requires the `tools` feature.
///
/// ```ignore
/// use ollama_oxide::{ChatRequest, ChatMessage, ToolDefinition};
/// use serde_json::json;
///
/// let request = ChatRequest::new("qwen3:0.6b", [
///     ChatMessage::user("What's the weather in Paris?")
/// ]).with_tools(vec![
///     ToolDefinition::function("get_weather", json!({
///         "type": "object",
///         "properties": {"location": {"type": "string"}},
///         "required": ["location"]
///     }))
/// ]);
/// ```
///
/// ## Multi-turn Conversation
///
/// ```no_run
/// use ollama_oxide::{ChatRequest, ChatMessage};
///
/// let request = ChatRequest::new("qwen3:0.6b", [
///     ChatMessage::system("You are a helpful assistant."),
///     ChatMessage::user("What is Rust?"),
///     ChatMessage::assistant("Rust is a systems programming language."),
///     ChatMessage::user("What are its main features?"),
/// ]);
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatRequest {
    /// Name of the model to use.
    pub model: String,

    /// Conversation history as an array of messages.
    ///
    /// Messages should be in chronological order, typically starting
    /// with an optional system message, followed by alternating user
    /// and assistant messages.
    pub messages: Vec<ChatMessage>,

    /// Optional list of function tools the model may call.
    ///
    /// When provided, the model may choose to call one or more of these
    /// functions instead of (or in addition to) generating a text response.
    ///
    /// Requires the `tools` feature.
    #[cfg(feature = "tools")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolDefinition>>,

    /// Output format (string like "json" or JSON schema object).
    ///
    /// Use `FormatSetting::json()` for JSON output, or provide a
    /// JSON Schema for structured output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<FormatSetting>,

    /// Runtime options for generation.
    ///
    /// Controls temperature, top_k, top_p, context length, etc.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<ModelOptions>,

    /// Whether to stream the response.
    ///
    /// `ChatRequest::new` sets this to `false` for non-streaming use.
    /// [`OllamaApiAsync::chat_stream`](crate::OllamaApiAsync::chat_stream) and
    /// [`OllamaApiSync::chat_stream_blocking`](crate::OllamaApiSync::chat_stream_blocking)
    /// clone the request and set `stream` to `true` regardless of this field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,

    /// Control thinking/reasoning output.
    ///
    /// When enabled, the model will include its reasoning process
    /// in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub think: Option<ThinkSetting>,

    /// How long to keep the model loaded in memory.
    ///
    /// Use `KeepAliveSetting::duration("5m")` for time-based,
    /// or `KeepAliveSetting::seconds(0)` to unload immediately.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_alive: Option<KeepAliveSetting>,

    /// Whether to return log probabilities for tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<bool>,

    /// Number of top log probabilities to return.
    ///
    /// Only used when `logprobs` is true.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<i32>,
}

impl ChatRequest {
    /// Create a new chat request.
    ///
    /// Creates a non-streaming request with the specified model and messages.
    /// The `stream` field is automatically set to `false`.
    ///
    /// # Arguments
    ///
    /// * `model` - Name of the model to use (e.g., "qwen3:0.6b", "llama3:8b")
    /// * `messages` - Conversation history as an iterator of messages
    ///
    /// # Examples
    ///
    /// ## With Vec
    ///
    /// ```no_run
    /// use ollama_oxide::{ChatRequest, ChatMessage};
    ///
    /// let messages = vec![
    ///     ChatMessage::user("Hello!")
    /// ];
    /// let request = ChatRequest::new("qwen3:0.6b", messages);
    /// ```
    ///
    /// ## With Array
    ///
    /// ```no_run
    /// use ollama_oxide::{ChatRequest, ChatMessage};
    ///
    /// let request = ChatRequest::new("qwen3:0.6b", [
    ///     ChatMessage::system("Be concise."),
    ///     ChatMessage::user("Hello!")
    /// ]);
    /// ```
    ///
    /// ## With Iterator
    ///
    /// ```no_run
    /// use ollama_oxide::{ChatRequest, ChatMessage};
    ///
    /// let user_msgs = ["Hi", "How are you?"];
    /// let messages = user_msgs.iter().map(|s| ChatMessage::user(*s));
    /// let request = ChatRequest::new("qwen3:0.6b", messages);
    /// ```
    pub fn new<M, I>(model: M, messages: I) -> Self
    where
        M: Into<String>,
        I: IntoIterator<Item = ChatMessage>,
    {
        Self {
            model: model.into(),
            messages: messages.into_iter().collect(),
            #[cfg(feature = "tools")]
            tools: None,
            format: None,
            options: None,
            stream: Some(false), // Non-streaming for v0.1.0
            think: None,
            keep_alive: None,
            logprobs: None,
            top_logprobs: None,
        }
    }

    /// Add a message to the conversation.
    ///
    /// Appends the message to the end of the conversation history.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to add
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ollama_oxide::{ChatRequest, ChatMessage};
    ///
    /// let request = ChatRequest::new("qwen3:0.6b", [
    ///     ChatMessage::system("Be helpful.")
    /// ]).with_message(ChatMessage::user("Hello!"));
    /// ```
    pub fn with_message(mut self, message: ChatMessage) -> Self {
        self.messages.push(message);
        self
    }

    /// Set whether the server should stream the response (NDJSON).
    ///
    /// For typical use, prefer [`OllamaApiAsync::chat_stream`](crate::OllamaApiAsync::chat_stream)
    /// or [`OllamaApiSync::chat_stream_blocking`](crate::OllamaApiSync::chat_stream_blocking),
    /// which send `stream: true` automatically.
    ///
    /// # Arguments
    ///
    /// * `stream` - `true` for streaming, `false` for a single JSON response.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ollama_oxide::{ChatRequest, ChatMessage};
    ///
    /// let request = ChatRequest::new("qwen3:0.6b", [ChatMessage::user("Hi!")]).with_stream(true);
    /// assert_eq!(request.stream, Some(true));
    /// ```
    pub fn with_stream(mut self, stream: bool) -> Self {
        self.stream = Some(stream);
        self
    }

    /// Set tools for function calling.
    ///
    /// Replaces any existing tools with the provided list.
    ///
    /// Requires the `tools` feature.
    ///
    /// # Arguments
    ///
    /// * `tools` - Vector of tool definitions
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ollama_oxide::{ChatRequest, ChatMessage, ToolDefinition};
    /// use serde_json::json;
    ///
    /// let request = ChatRequest::new("qwen3:0.6b", [
    ///     ChatMessage::user("What time is it?")
    /// ]).with_tools(vec![
    ///     ToolDefinition::function_no_params("get_time")
    ///         .with_description("Get the current time")
    /// ]);
    /// ```
    #[cfg(feature = "tools")]
    pub fn with_tools(mut self, tools: Vec<ToolDefinition>) -> Self {
        self.tools = Some(tools);
        self
    }

    /// Add a single tool.
    ///
    /// Appends a tool to the existing list (creating the list if needed).
    ///
    /// Requires the `tools` feature.
    ///
    /// # Arguments
    ///
    /// * `tool` - The tool definition to add
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ollama_oxide::{ChatRequest, ChatMessage, ToolDefinition};
    /// use serde_json::json;
    ///
    /// let request = ChatRequest::new("qwen3:0.6b", [
    ///     ChatMessage::user("Search and calculate")
    /// ])
    /// .with_tool(ToolDefinition::function("search", json!({})))
    /// .with_tool(ToolDefinition::function("calculate", json!({})));
    /// ```
    #[cfg(feature = "tools")]
    pub fn with_tool(mut self, tool: ToolDefinition) -> Self {
        self.tools.get_or_insert_with(Vec::new).push(tool);
        self
    }

    /// Set the output format.
    ///
    /// # Arguments
    ///
    /// * `format` - The format setting
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ollama_oxide::{ChatRequest, ChatMessage, FormatSetting};
    ///
    /// let request = ChatRequest::new("qwen3:0.6b", [
    ///     ChatMessage::user("List 3 colors as JSON")
    /// ]).with_format(FormatSetting::json());
    /// ```
    pub fn with_format(mut self, format: impl Into<FormatSetting>) -> Self {
        self.format = Some(format.into());
        self
    }

    /// Set model options.
    ///
    /// # Arguments
    ///
    /// * `options` - The model options
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ollama_oxide::{ChatRequest, ChatMessage, ModelOptions};
    ///
    /// let request = ChatRequest::new("qwen3:0.6b", [
    ///     ChatMessage::user("Be creative!")
    /// ]).with_options(ModelOptions::default()
    ///     .with_temperature(0.9)
    ///     .with_top_p(0.95));
    /// ```
    pub fn with_options(mut self, options: ModelOptions) -> Self {
        self.options = Some(options);
        self
    }

    /// Set the think option.
    ///
    /// When enabled, the model will include reasoning output.
    ///
    /// # Arguments
    ///
    /// * `think` - The think setting
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ollama_oxide::{ChatRequest, ChatMessage, ThinkSetting};
    ///
    /// let request = ChatRequest::new("qwen3:0.6b", [
    ///     ChatMessage::user("Solve: 15 * 7 + 23")
    /// ]).with_think(ThinkSetting::enabled());
    /// ```
    pub fn with_think(mut self, think: impl Into<ThinkSetting>) -> Self {
        self.think = Some(think.into());
        self
    }

    /// Set the keep_alive duration.
    ///
    /// # Arguments
    ///
    /// * `keep_alive` - How long to keep the model loaded
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ollama_oxide::{ChatRequest, ChatMessage, KeepAliveSetting};
    ///
    /// let request = ChatRequest::new("qwen3:0.6b", [
    ///     ChatMessage::user("Hello")
    /// ]).with_keep_alive(KeepAliveSetting::duration("10m"));
    /// ```
    pub fn with_keep_alive(mut self, keep_alive: impl Into<KeepAliveSetting>) -> Self {
        self.keep_alive = Some(keep_alive.into());
        self
    }

    /// Enable log probabilities.
    ///
    /// # Arguments
    ///
    /// * `logprobs` - Whether to return log probabilities
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ollama_oxide::{ChatRequest, ChatMessage};
    ///
    /// let request = ChatRequest::new("qwen3:0.6b", [
    ///     ChatMessage::user("Hello")
    /// ]).with_logprobs(true);
    /// ```
    pub fn with_logprobs(mut self, logprobs: bool) -> Self {
        self.logprobs = Some(logprobs);
        self
    }

    /// Set number of top log probabilities to return.
    ///
    /// # Arguments
    ///
    /// * `n` - Number of top log probabilities
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ollama_oxide::{ChatRequest, ChatMessage};
    ///
    /// let request = ChatRequest::new("qwen3:0.6b", [
    ///     ChatMessage::user("Hello")
    /// ]).with_logprobs(true)
    ///   .with_top_logprobs(5);
    /// ```
    pub fn with_top_logprobs(mut self, n: i32) -> Self {
        self.top_logprobs = Some(n);
        self
    }

    /// Get the model name.
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Get the messages.
    pub fn messages(&self) -> &[ChatMessage] {
        &self.messages
    }

    /// Get the number of messages.
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    /// Check if any tools are defined.
    ///
    /// Requires the `tools` feature.
    #[cfg(feature = "tools")]
    pub fn has_tools(&self) -> bool {
        self.tools.as_ref().map(|t| !t.is_empty()).unwrap_or(false)
    }

    /// Get the tools if any.
    ///
    /// Requires the `tools` feature.
    #[cfg(feature = "tools")]
    pub fn tools(&self) -> Option<&[ToolDefinition]> {
        self.tools.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[allow(unused)]
    use serde_json::json;

    #[test]
    fn test_chat_request_new_with_vec() {
        let request = ChatRequest::new("qwen3:0.6b", vec![ChatMessage::user("Hello")]);

        assert_eq!(request.model, "qwen3:0.6b");
        assert_eq!(request.messages.len(), 1);
        assert_eq!(request.stream, Some(false));
    }

    #[test]
    fn test_chat_request_new_with_array() {
        let request = ChatRequest::new(
            "qwen3:0.6b",
            [
                ChatMessage::system("Be helpful"),
                ChatMessage::user("Hello"),
            ],
        );

        assert_eq!(request.messages.len(), 2);
        assert!(request.messages[0].is_system());
        assert!(request.messages[1].is_user());
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
            .with_message(ChatMessage::assistant("2"))
            .with_message(ChatMessage::user("3"));

        assert_eq!(request.messages.len(), 3);
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

        assert!(request.has_tools());
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

        assert!(request.options.is_some());
        assert_eq!(request.options.unwrap().temperature, Some(0.7));
    }

    #[test]
    fn test_chat_request_with_think() {
        let request = ChatRequest::new("model", [ChatMessage::user("Hi")])
            .with_think(ThinkSetting::enabled());

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
    fn test_chat_request_model() {
        let request = ChatRequest::new("my-model", [ChatMessage::user("Hi")]);
        assert_eq!(request.model(), "my-model");
    }

    #[test]
    fn test_chat_request_messages() {
        let request = ChatRequest::new(
            "model",
            [ChatMessage::user("1"), ChatMessage::assistant("2")],
        );

        assert_eq!(request.messages().len(), 2);
        assert_eq!(request.message_count(), 2);
    }

    #[cfg(feature = "tools")]
    #[test]
    fn test_chat_request_has_tools() {
        let without = ChatRequest::new("model", [ChatMessage::user("Hi")]);
        assert!(!without.has_tools());

        let with = without.with_tool(ToolDefinition::function("f", json!({})));
        assert!(with.has_tools());
    }

    #[test]
    fn test_chat_request_serialize() {
        let request = ChatRequest::new("qwen3:0.6b", [ChatMessage::user("Hello")]);

        let json = serde_json::to_value(&request).unwrap();

        assert_eq!(json["model"], "qwen3:0.6b");
        assert_eq!(json["messages"][0]["role"], "user");
        assert_eq!(json["messages"][0]["content"], "Hello");
        assert_eq!(json["stream"], false);
        // Optional fields should be omitted
        #[cfg(feature = "tools")]
        assert!(json.get("tools").is_none());
        assert!(json.get("format").is_none());
    }

    #[cfg(feature = "tools")]
    #[test]
    fn test_chat_request_serialize_with_tools() {
        let request = ChatRequest::new("model", [ChatMessage::user("Hi")]).with_tools(vec![
            ToolDefinition::function(
                "get_weather",
                json!({
                    "type": "object",
                    "properties": {"location": {"type": "string"}}
                }),
            )
            .with_description("Get weather"),
        ]);

        let json = serde_json::to_value(&request).unwrap();

        assert!(json.get("tools").is_some());
        assert_eq!(json["tools"][0]["type"], "function");
        assert_eq!(json["tools"][0]["function"]["name"], "get_weather");
    }

    #[test]
    fn test_chat_request_serialize_full() {
        let request = ChatRequest::new(
            "qwen3:0.6b",
            [
                ChatMessage::system("Be helpful."),
                ChatMessage::user("What's 2+2?"),
            ],
        )
        .with_format(FormatSetting::json())
        .with_options(ModelOptions::default().with_temperature(0.7))
        .with_think(ThinkSetting::enabled())
        .with_keep_alive(KeepAliveSetting::duration("5m"))
        .with_logprobs(true)
        .with_top_logprobs(3);

        let json = serde_json::to_value(&request).unwrap();

        assert_eq!(json["model"], "qwen3:0.6b");
        assert_eq!(json["messages"].as_array().unwrap().len(), 2);
        assert_eq!(json["stream"], false);
        assert_eq!(json["format"], "json");
        // Check temperature is approximately 0.7 (f32 precision)
        let temp = json["options"]["temperature"].as_f64().unwrap();
        assert!((temp - 0.7).abs() < 0.001, "Expected ~0.7, got {}", temp);
        assert_eq!(json["think"], true);
        assert_eq!(json["keep_alive"], "5m");
        assert_eq!(json["logprobs"], true);
        assert_eq!(json["top_logprobs"], 3);
    }

    #[test]
    fn test_chat_request_deserialize() {
        let json = r#"{
            "model": "qwen3:0.6b",
            "messages": [
                {"role": "user", "content": "Hello"}
            ],
            "stream": false
        }"#;

        let request: ChatRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.model, "qwen3:0.6b");
        assert_eq!(request.messages.len(), 1);
        assert_eq!(request.stream, Some(false));
    }

    #[test]
    fn test_chat_request_clone() {
        #[cfg(feature = "tools")]
        let request = ChatRequest::new("model", [ChatMessage::user("Hi")])
            .with_tool(ToolDefinition::function("f", json!({})));
        #[cfg(not(feature = "tools"))]
        let request = ChatRequest::new("model", [ChatMessage::user("Hi")]);

        let cloned = request.clone();
        assert_eq!(request, cloned);
    }

    #[test]
    fn test_chat_request_equality() {
        let req1 = ChatRequest::new("model", [ChatMessage::user("Hi")]);
        let req2 = ChatRequest::new("model", [ChatMessage::user("Hi")]);
        let req3 = ChatRequest::new("model", [ChatMessage::user("Bye")]);

        assert_eq!(req1, req2);
        assert_ne!(req1, req3);
    }

    #[cfg(feature = "tools")]
    #[test]
    fn test_chat_request_matches_api_format() {
        // Test that our serialization matches the expected Ollama API format
        let request = ChatRequest::new(
            "qwen3:0.6b",
            [
                ChatMessage::system("You are a helpful assistant."),
                ChatMessage::user("What's the weather in Paris?"),
            ],
        )
        .with_tools(vec![
            ToolDefinition::function(
                "get_weather",
                json!({
                    "type": "object",
                    "properties": {
                        "location": {"type": "string"}
                    },
                    "required": ["location"]
                }),
            )
            .with_description("Get the current weather for a location"),
        ]);

        let json_value = serde_json::to_value(&request).unwrap();
        let json_string = serde_json::to_string_pretty(&json_value).unwrap();

        // Verify structure matches API docs
        assert!(json_string.contains("\"model\": \"qwen3:0.6b\""));
        assert!(json_string.contains("\"stream\": false"));
        assert!(json_string.contains("\"messages\""));
        assert!(json_string.contains("\"tools\""));
        assert!(json_string.contains("\"type\": \"function\""));
    }
}
