# Proposal: Ergonomic Tools API for ollama-oxide

**Document:** Design Proposal
**Version:** 1.0
**Created:** 2026-01-24
**Status:** Draft for Review

## Problem Statement

The current tool/function calling design in the chat implementation plan requires developers to:

1. **Manually construct JSON schemas** - Verbose and error-prone
2. **Parse tool calls manually** - Extract function names and arguments
3. **Dispatch to handlers manually** - Match names and route to functions
4. **Handle type conversions** - Convert JSON to Rust types

### Current Approach (Verbose)

```rust
// 1. Define tool with raw JSON schema (verbose, error-prone)
let weather_tool = ToolDefinition::function(
    "get_weather",
    json!({
        "type": "object",
        "properties": {
            "location": {
                "type": "string",
                "description": "City name"
            },
            "unit": {
                "type": "string",
                "enum": ["celsius", "fahrenheit"]
            }
        },
        "required": ["location"]
    })
).with_description("Get weather for a location");

// 2. Handle response (manual dispatch)
if response.has_tool_calls() {
    for call in response.tool_calls().unwrap() {
        let name = call.function_name().unwrap();
        let args = call.arguments().unwrap();

        // Manual dispatch
        match name {
            "get_weather" => {
                // Manual type extraction
                let location = args["location"].as_str().unwrap();
                let unit = args.get("unit").and_then(|v| v.as_str());
                // Call handler...
            }
            _ => {}
        }
    }
}
```

**Pain Points:**
- JSON schema is verbose and must match Rust types manually
- No compile-time validation of schema correctness
- Manual string matching for dispatch
- Manual JSON-to-Rust type conversion
- Easy to make mistakes

---

## Research Findings

### Industry Patterns

Based on research of existing Rust LLM libraries:

| Library | Pattern | Key Features |
|---------|---------|--------------|
| [tools-rs](https://crates.io/crates/tools-rs) | `#[tool]` macro | Auto-schema, registry, async support |
| [rig-core](https://docs.rs/rig-core/latest/rig/) | `Tool` trait + macro | Trait-based, agent integration |
| [schemars](https://graham.cool/schemars/) | `#[derive(JsonSchema)]` | Auto JSON schema from types |
| [llm-connector](https://lib.rs/crates/llm-connector) | Provider abstraction | Multi-backend, streaming |

### Key Insights

1. **Schemars** can auto-generate JSON schemas from Rust structs
2. **Trait-based design** provides flexibility without macros
3. **Registry pattern** enables automatic dispatch
4. **Derive macros** offer convenience for common cases

---

## Proposed Design: Three-Tier API

### Tier 1: Low-Level (Current API - Keep As-Is)

The existing `ToolDefinition`, `ToolCall`, etc. remain for advanced users who need full control.

```rust
// Still works for power users
let tool = ToolDefinition::function("my_tool", json!({...}));
```

### Tier 2: Type-Safe Tools (New - Trait-Based)

A `Tool` trait with automatic schema generation via schemars.

```rust
use ollama_oxide::tools::{Tool, ToolResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Define parameters as a struct
#[derive(Debug, Deserialize, JsonSchema)]
struct GetWeatherParams {
    /// City name (e.g., "Paris", "New York")
    location: String,

    /// Temperature unit
    #[serde(default)]
    unit: Option<TemperatureUnit>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
enum TemperatureUnit {
    Celsius,
    Fahrenheit,
}

/// Define the tool
struct GetWeather;

impl Tool for GetWeather {
    type Params = GetWeatherParams;
    type Output = WeatherResult;

    fn name(&self) -> &'static str {
        "get_weather"
    }

    fn description(&self) -> &'static str {
        "Get the current weather for a location"
    }

    async fn execute(&self, params: Self::Params) -> ToolResult<Self::Output> {
        // Type-safe params - no manual JSON parsing!
        let weather = fetch_weather(&params.location, params.unit).await?;
        Ok(weather)
    }
}
```

**Benefits:**
- JSON schema auto-generated from `GetWeatherParams`
- Doc comments become parameter descriptions
- Type-safe execution with `Self::Params`
- Compile-time validation

### Tier 3: Tool Registry (New - Automatic Dispatch)

A registry that collects tools and handles dispatch automatically.

```rust
use ollama_oxide::tools::{ToolRegistry, Tool};

// Create registry and register tools
let mut registry = ToolRegistry::new();
registry.register(GetWeather);
registry.register(SearchWeb);
registry.register(Calculator);

// Get tool definitions for ChatRequest (auto-generated)
let tools = registry.definitions(); // Vec<ToolDefinition>

// Create request with tools
let request = ChatRequest::new("qwen3:0.6b", messages)
    .with_tool_definitions(&registry); // or .with_tools(registry.definitions())

let response = client.chat(&request).await?;

// Automatic dispatch - no manual matching!
if response.has_tool_calls() {
    for call in response.tool_calls().unwrap() {
        let result = registry.execute(call).await?;
        println!("Tool result: {:?}", result);
    }
}
```

---

## Detailed Type Definitions

### Tool Trait

```rust
/// Trait for defining type-safe tools
pub trait Tool: Send + Sync {
    /// The parameter type (must derive JsonSchema + Deserialize)
    type Params: for<'de> Deserialize<'de> + JsonSchema + Send;

    /// The output type (must derive Serialize)
    type Output: Serialize + Send;

    /// Tool name (used in function calls)
    fn name(&self) -> &'static str;

    /// Human-readable description
    fn description(&self) -> &'static str;

    /// Execute the tool with parsed parameters
    fn execute(&self, params: Self::Params) -> impl Future<Output = ToolResult<Self::Output>> + Send;

    /// Generate the JSON schema for parameters (auto-implemented)
    fn parameters_schema(&self) -> serde_json::Value {
        let schema = schemars::schema_for!(Self::Params);
        serde_json::to_value(schema).unwrap()
    }

    /// Convert to ToolDefinition (auto-implemented)
    fn to_definition(&self) -> ToolDefinition {
        ToolDefinition::function(self.name(), self.parameters_schema())
            .with_description(self.description())
    }
}
```

### ToolResult Type

```rust
/// Result type for tool execution
pub type ToolResult<T> = Result<T, ToolError>;

/// Errors that can occur during tool execution
#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    #[error("Invalid parameters: {0}")]
    InvalidParams(String),

    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Tool not found: {0}")]
    NotFound(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(#[from] serde_json::Error),
}
```

### ToolRegistry

```rust
/// Registry for collecting and dispatching tools
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn ErasedTool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self { tools: HashMap::new() }
    }

    /// Register a tool
    pub fn register<T: Tool + 'static>(&mut self, tool: T) {
        self.tools.insert(tool.name().to_string(), Box::new(tool));
    }

    /// Get all tool definitions (for ChatRequest)
    pub fn definitions(&self) -> Vec<ToolDefinition> {
        self.tools.values().map(|t| t.to_definition()).collect()
    }

    /// Execute a tool call
    pub async fn execute(&self, call: &ToolCall) -> ToolResult<serde_json::Value> {
        let name = call.function_name()
            .ok_or_else(|| ToolError::InvalidParams("Missing function name".into()))?;

        let tool = self.tools.get(name)
            .ok_or_else(|| ToolError::NotFound(name.to_string()))?;

        let args = call.arguments()
            .cloned()
            .unwrap_or_else(|| json!({}));

        tool.execute_erased(args).await
    }

    /// Execute all tool calls from a response
    pub async fn execute_all(&self, response: &ChatResponse) -> Vec<ToolResult<serde_json::Value>> {
        let mut results = Vec::new();
        if let Some(calls) = response.tool_calls() {
            for call in calls {
                results.push(self.execute(call).await);
            }
        }
        results
    }

    /// Execute a tool call (blocking/sync version)
    pub fn execute_blocking(&self, call: &ToolCall) -> ToolResult<serde_json::Value> {
        tokio::runtime::Handle::current().block_on(self.execute(call))
    }

    /// Execute all tool calls from a response (blocking/sync version)
    pub fn execute_all_blocking(&self, response: &ChatResponse) -> Vec<ToolResult<serde_json::Value>> {
        tokio::runtime::Handle::current().block_on(self.execute_all(response))
    }
}

// Type-erased trait for storing heterogeneous tools
trait ErasedTool: Send + Sync {
    fn name(&self) -> &'static str;
    fn to_definition(&self) -> ToolDefinition;
    fn execute_erased(&self, args: serde_json::Value) -> BoxFuture<'_, ToolResult<serde_json::Value>>;
}

impl<T: Tool + 'static> ErasedTool for T {
    fn name(&self) -> &'static str {
        Tool::name(self)
    }

    fn to_definition(&self) -> ToolDefinition {
        Tool::to_definition(self)
    }

    fn execute_erased(&self, args: serde_json::Value) -> BoxFuture<'_, ToolResult<serde_json::Value>> {
        Box::pin(async move {
            let params: Self::Params = serde_json::from_value(args)?;
            let output = self.execute(params).await?;
            Ok(serde_json::to_value(output)?)
        })
    }
}
```

---

## Complete Example: Smart Home with Type-Safe Tools

```rust
use ollama_oxide::{
    ChatMessage, ChatRequest, OllamaApiAsync, OllamaClient,
    tools::{Tool, ToolRegistry, ToolResult, ToolError},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════════
// STEP 1: Define parameter structs (schemas auto-generated!)
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize, JsonSchema)]
struct LightParams {
    /// Room name (e.g., "living room", "bedroom")
    room: String,

    /// Light action
    action: LightAction,

    /// Brightness percentage (0-100)
    #[serde(default)]
    brightness: Option<u8>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
enum LightAction {
    On,
    Off,
    Toggle,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ThermostatParams {
    /// Target temperature in Celsius
    temperature: f32,

    /// HVAC mode
    #[serde(default)]
    mode: Option<HvacMode>,
}

#[derive(Debug, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "lowercase")]
enum HvacMode {
    Heat,
    Cool,
    #[default]
    Auto,
    Off,
}

// ═══════════════════════════════════════════════════════════════════
// STEP 2: Define output types
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Serialize)]
struct LightResult {
    success: bool,
    message: String,
}

#[derive(Debug, Serialize)]
struct ThermostatResult {
    success: bool,
    current_temp: f32,
    target_temp: f32,
}

// ═══════════════════════════════════════════════════════════════════
// STEP 3: Implement Tool trait (clean, minimal boilerplate)
// ═══════════════════════════════════════════════════════════════════

struct ControlLights;

impl Tool for ControlLights {
    type Params = LightParams;
    type Output = LightResult;

    fn name(&self) -> &'static str { "control_lights" }
    fn description(&self) -> &'static str { "Control smart lights in a room" }

    async fn execute(&self, params: Self::Params) -> ToolResult<Self::Output> {
        // params is already typed! No JSON parsing needed.
        println!("Controlling lights in {}: {:?}", params.room, params.action);

        Ok(LightResult {
            success: true,
            message: format!(
                "Lights in {} turned {:?}{}",
                params.room,
                params.action,
                params.brightness.map(|b| format!(" at {}%", b)).unwrap_or_default()
            ),
        })
    }
}

struct ControlThermostat;

impl Tool for ControlThermostat {
    type Params = ThermostatParams;
    type Output = ThermostatResult;

    fn name(&self) -> &'static str { "control_thermostat" }
    fn description(&self) -> &'static str { "Set thermostat temperature and mode" }

    async fn execute(&self, params: Self::Params) -> ToolResult<Self::Output> {
        println!("Setting thermostat to {}°C", params.temperature);

        Ok(ThermostatResult {
            success: true,
            current_temp: 21.0,
            target_temp: params.temperature,
        })
    }
}

// ═══════════════════════════════════════════════════════════════════
// STEP 4: Use with ToolRegistry (automatic dispatch!)
// ═══════════════════════════════════════════════════════════════════

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OllamaClient::default()?;

    // Create registry and register tools
    let mut registry = ToolRegistry::new();
    registry.register(ControlLights);
    registry.register(ControlThermostat);

    // Build request with auto-generated tool definitions
    let request = ChatRequest::new("qwen3:0.6b", vec![
        ChatMessage::system("You are a smart home assistant."),
        ChatMessage::user("Turn on the living room lights at 50%"),
    ])
    .with_tools(registry.definitions()); // Auto-generated schemas!

    let response = client.chat(&request).await?;

    // Automatic dispatch - no manual matching!
    if response.has_tool_calls() {
        let results = registry.execute_all(&response).await;
        for result in results {
            match result {
                Ok(value) => println!("✓ {}", value),
                Err(e) => println!("✗ Error: {}", e),
            }
        }
    }

    Ok(())
}
```

---

## Comparison: Before vs After

### Before (Current Design)

```rust
// 40+ lines of JSON schema definition
let tool = ToolDefinition::function("control_lights", json!({
    "type": "object",
    "properties": {
        "room": { "type": "string", "description": "Room name" },
        "action": { "type": "string", "enum": ["on", "off", "toggle"] },
        "brightness": { "type": "integer", "minimum": 0, "maximum": 100 }
    },
    "required": ["room", "action"]
})).with_description("Control lights");

// Manual dispatch with string matching
match call.function_name() {
    Some("control_lights") => {
        let room = args["room"].as_str().unwrap();
        let action = args["action"].as_str().unwrap();
        // ... more manual parsing
    }
    _ => {}
}
```

### After (Proposed Design)

```rust
// Type-safe parameter definition (schema auto-generated!)
#[derive(Deserialize, JsonSchema)]
struct LightParams {
    room: String,
    action: LightAction,
    brightness: Option<u8>,
}

// Clean tool implementation
impl Tool for ControlLights {
    type Params = LightParams;
    type Output = LightResult;

    fn name(&self) -> &'static str { "control_lights" }
    fn description(&self) -> &'static str { "Control lights" }

    async fn execute(&self, params: Self::Params) -> ToolResult<Self::Output> {
        // params.room, params.action, params.brightness - all typed!
    }
}

// Automatic dispatch
let results = registry.execute_all(&response).await;
```

---

## Implementation Phases

### Phase 1: Core Types (v0.1.0)
- Keep current `ToolDefinition`, `ToolCall` as-is
- Add `Tool` trait
- Add `ToolResult`, `ToolError`
- Add `ToolRegistry` with async methods (`execute`, `execute_all`)
- Add sync dispatch methods (`execute_blocking`, `execute_all_blocking`)
- No proc-macros (just trait-based)

### Phase 2: Convenience Features (v0.1.x)
- Add `ChatRequest::with_tool_registry()` method
- Add `ChatResponse::execute_tools()` helper

### Phase 3: Macro Support (v0.2.0 or later, optional crate)
- `#[tool]` attribute macro for even less boilerplate
- `#[derive(Tool)]` for simple cases

---

## Feature Flag: `tools`

The ergonomic tools API is behind an **optional feature flag** (not enabled by default):

```toml
[features]
default = []
tools = ["dep:schemars", "dep:futures"]

[dependencies]
schemars = { version = "0.8", optional = true }
futures = { version = "0.3", optional = true }
```

**Usage:**
```toml
# Without ergonomic tools (default)
ollama-oxide = "0.1"

# With ergonomic tools
ollama-oxide = { version = "0.1", features = ["tools"] }
```

**Rationale:**
- Keeps core library lightweight for users who only need low-level types
- `schemars` and `futures` are lightweight but unnecessary for basic usage
- Clear opt-in for advanced functionality

---

## Benefits Summary

| Aspect | Current | Proposed |
|--------|---------|----------|
| **Schema Definition** | Manual JSON | Auto-generated from types |
| **Type Safety** | Runtime JSON parsing | Compile-time validation |
| **Dispatch** | Manual string matching | Automatic via registry |
| **Boilerplate** | ~40 lines per tool | ~15 lines per tool |
| **Error Handling** | Ad-hoc | Structured `ToolError` |
| **Discoverability** | None | Registry introspection |

---

## Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| `Tool::execute` async by default | **Yes** | Modern async-first design; sync can use `block_on` |
| Parameter structs require `Debug` | **Yes** | Better error messages when deserialization fails |
| `ToolRegistry` thread-safe | **Yes** | Use `Arc<RwLock<...>>` for safe concurrent access |
| Sync dispatch methods in Phase 1 | **Yes** | Consistency with `OllamaApiSync`; low implementation cost |
| Feature flag (not default) | **Yes** | Keeps core library lightweight; opt-in for advanced features |

---

## References

- [tools-rs](https://crates.io/crates/tools-rs) - Rust tool registration pattern
- [rig-core](https://docs.rs/rig-core/latest/rig/) - LLM agent framework with tools
- [schemars](https://graham.cool/schemars/) - JSON Schema generation
- [OpenAI Function Calling](https://platform.openai.com/docs/guides/function-calling) - Industry standard
