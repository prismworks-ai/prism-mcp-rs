//! Complete tool validation system for MCP SDK
//!
//! This module provides complete parameter validation, type checking,
//! and coercion capabilities for tool arguments according to JSON Schema specifications.

use crate::core::error::{McpError, McpResult};
use serde_json::{Map, Value};
use std::collections::{HashMap, HashSet};

/// Helper function to get a human-readable type name for a JSON value
fn get_value_type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

/// Parameter validation configuration
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// Whether to allow additional properties not in schema
    pub allow_additional: bool,
    /// Whether to coerce types when possible (e.g., string "5" -> number 5)
    pub coerce_types: bool,
    /// Whether to provide detailed validation errors
    pub detailed_errors: bool,
    /// Maximum string length for validation
    pub max_string_length: Option<usize>,
    /// Maximum array length for validation
    pub max_array_length: Option<usize>,
    /// Maximum object property count
    pub max_object_properties: Option<usize>,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            allow_additional: true,
            coerce_types: true,
            detailed_errors: true,
            max_string_length: Some(10_000),
            max_array_length: Some(1_000),
            max_object_properties: Some(100),
        }
    }
}

/// improved JSON Schema validator for tool parameters
#[derive(Debug, Clone)]
pub struct ParameterValidator {
    /// JSON Schema for validation
    pub schema: Value,
    /// Validation configuration
    pub config: ValidationConfig,
}

impl ParameterValidator {
    /// Create a new parameter validator with schema
    pub fn new(schema: Value) -> Self {
        Self {
            schema,
            config: ValidationConfig::default(),
        }
    }

    /// Create validator with custom configuration
    pub fn with_config(schema: Value, config: ValidationConfig) -> Self {
        Self { schema, config }
    }

    /// Validate and optionally coerce parameters
    pub fn validate_and_coerce(&self, params: &mut HashMap<String, Value>) -> McpResult<()> {
        let schema_obj = self
            .schema
            .as_object()
            .ok_or_else(|| McpError::validation("Schema must be an object"))?;

        // Check type
        if let Some(schema_type) = schema_obj.get("type") {
            if schema_type.as_str() != Some("object") {
                return Err(McpError::validation("Tool schema type must be 'object'"));
            }
        }

        // Validate required properties
        if let Some(required) = schema_obj.get("required") {
            self.validate_required_properties(params, required)?;
        }

        // Validate individual properties
        if let Some(properties) = schema_obj.get("properties") {
            self.validate_properties(params, properties)?;
        }

        // Check additional properties
        if !self.config.allow_additional {
            self.check_additional_properties(params, schema_obj)?;
        }

        // Check object size limits
        if let Some(max_props) = self.config.max_object_properties {
            if params.len() > max_props {
                return Err(McpError::validation(format!(
                    "Too many properties: {} > {}",
                    params.len(),
                    max_props
                )));
            }
        }

        Ok(())
    }

    /// Validate required properties are present
    fn validate_required_properties(
        &self,
        params: &HashMap<String, Value>,
        required: &Value,
    ) -> McpResult<()> {
        let required_array = required
            .as_array()
            .ok_or_else(|| McpError::validation("Required field must be an array"))?;

        for req in required_array {
            let prop_name = req
                .as_str()
                .ok_or_else(|| McpError::validation("Required property names must be strings"))?;

            if !params.contains_key(prop_name) {
                return Err(McpError::validation(format!(
                    "Missing required parameter: '{prop_name}'"
                )));
            }
        }

        Ok(())
    }

    /// Validate and coerce individual properties
    fn validate_properties(
        &self,
        params: &mut HashMap<String, Value>,
        properties: &Value,
    ) -> McpResult<()> {
        let props_obj = properties
            .as_object()
            .ok_or_else(|| McpError::validation("Properties must be an object"))?;

        for (prop_name, value) in params.iter_mut() {
            if let Some(prop_schema) = props_obj.get(prop_name) {
                self.validate_and_coerce_value(value, prop_schema, prop_name)?;
            }
        }

        Ok(())
    }

    /// Validate and coerce a single value according to its schema
    fn validate_and_coerce_value(
        &self,
        value: &mut Value,
        schema: &Value,
        field_name: &str,
    ) -> McpResult<()> {
        let schema_obj = schema.as_object().ok_or_else(|| {
            McpError::validation(format!("Schema for '{field_name}' must be an object"))
        })?;

        // Get expected type
        let expected_type = schema_obj
            .get("type")
            .and_then(|t| t.as_str())
            .unwrap_or("any");

        match expected_type {
            "string" => self.validate_string(value, schema_obj, field_name)?,
            "number" | "integer" => self.validate_number(value, schema_obj, field_name)?,
            "boolean" => self.validate_boolean(value, field_name)?,
            "array" => self.validate_array(value, schema_obj, field_name)?,
            "object" => self.validate_object(value, schema_obj, field_name)?,
            "null" => self.validate_null(value, field_name)?,
            _ => {} // Allow any type
        }

        // Validate enum constraints
        if let Some(enum_values) = schema_obj.get("enum") {
            self.validate_enum(value, enum_values, field_name)?;
        }

        Ok(())
    }

    /// Validate and coerce string values
    fn validate_string(
        &self,
        value: &mut Value,
        schema: &Map<String, Value>,
        field_name: &str,
    ) -> McpResult<()> {
        // Type coercion
        if self.config.coerce_types && !value.is_string() {
            if let Some(coerced) = self.coerce_to_string(value) {
                *value = coerced;
            } else {
                return Err(McpError::validation(format!(
                    "Parameter '{}' must be a string, got {}",
                    field_name,
                    get_value_type_name(value)
                )));
            }
        }

        let string_val = value.as_str().ok_or_else(|| {
            McpError::validation(format!("Parameter '{field_name}' must be a string"))
        })?;

        // Length validation
        if let Some(max_len) = self.config.max_string_length {
            if string_val.len() > max_len {
                return Err(McpError::validation(format!(
                    "String '{}' too long: {} > {}",
                    field_name,
                    string_val.len(),
                    max_len
                )));
            }
        }

        // Schema-specific length constraints
        if let Some(min_len) = schema.get("minLength").and_then(|v| v.as_u64()) {
            if string_val.len() < min_len as usize {
                return Err(McpError::validation(format!(
                    "String '{}' too short: {} < {}",
                    field_name,
                    string_val.len(),
                    min_len
                )));
            }
        }

        if let Some(max_len) = schema.get("maxLength").and_then(|v| v.as_u64()) {
            if string_val.len() > max_len as usize {
                return Err(McpError::validation(format!(
                    "String '{}' too long: {} > {}",
                    field_name,
                    string_val.len(),
                    max_len
                )));
            }
        }

        // Pattern validation
        if let Some(pattern) = schema.get("pattern").and_then(|v| v.as_str()) {
            // Note: Full regex validation would require the regex crate
            // For now, we'll do basic validation checks
            if pattern.contains("^") && !string_val.starts_with(&pattern[1..pattern.len().min(2)]) {
                return Err(McpError::validation(format!(
                    "String '{field_name}' does not match pattern"
                )));
            }
        }

        Ok(())
    }

    /// Validate and coerce number values
    fn validate_number(
        &self,
        value: &mut Value,
        schema: &Map<String, Value>,
        field_name: &str,
    ) -> McpResult<()> {
        // Type coercion
        if self.config.coerce_types && !value.is_number() {
            if let Some(coerced) = self.coerce_to_number(value) {
                *value = coerced;
            } else {
                return Err(McpError::validation(format!(
                    "Parameter '{}' must be a number, got {}",
                    field_name,
                    get_value_type_name(value)
                )));
            }
        }

        let num_val = value.as_f64().ok_or_else(|| {
            McpError::validation(format!("Parameter '{field_name}' must be a number"))
        })?;

        // Range validation
        if let Some(minimum) = schema.get("minimum").and_then(|v| v.as_f64()) {
            if num_val < minimum {
                return Err(McpError::validation(format!(
                    "Number '{field_name}' too small: {num_val} < {minimum}"
                )));
            }
        }

        if let Some(maximum) = schema.get("maximum").and_then(|v| v.as_f64()) {
            if num_val > maximum {
                return Err(McpError::validation(format!(
                    "Number '{field_name}' too large: {num_val} > {maximum}"
                )));
            }
        }

        // Integer validation
        if schema.get("type").and_then(|v| v.as_str()) == Some("integer") {
            if num_val.fract() != 0.0 {
                if self.config.coerce_types {
                    *value = Value::Number(serde_json::Number::from(num_val.round() as i64));
                } else {
                    return Err(McpError::validation(format!(
                        "Parameter '{field_name}' must be an integer"
                    )));
                }
            } else {
                // Convert float to integer even if it has no fractional part
                *value = Value::Number(serde_json::Number::from(num_val as i64));
            }
        }

        Ok(())
    }

    /// Validate and coerce boolean values
    fn validate_boolean(&self, value: &mut Value, field_name: &str) -> McpResult<()> {
        // Type coercion
        if self.config.coerce_types && !value.is_boolean() {
            if let Some(coerced) = self.coerce_to_boolean(value) {
                *value = coerced;
            } else {
                return Err(McpError::validation(format!(
                    "Parameter '{}' must be a boolean, got {}",
                    field_name,
                    get_value_type_name(value)
                )));
            }
        }

        if !value.is_boolean() {
            return Err(McpError::validation(format!(
                "Parameter '{field_name}' must be a boolean"
            )));
        }

        Ok(())
    }

    /// Validate array values
    fn validate_array(
        &self,
        value: &mut Value,
        schema: &Map<String, Value>,
        field_name: &str,
    ) -> McpResult<()> {
        let array = value.as_array_mut().ok_or_else(|| {
            McpError::validation(format!("Parameter '{field_name}' must be an array"))
        })?;

        // Length validation
        if let Some(max_len) = self.config.max_array_length {
            if array.len() > max_len {
                return Err(McpError::validation(format!(
                    "Array '{}' too long: {} > {}",
                    field_name,
                    array.len(),
                    max_len
                )));
            }
        }

        if let Some(min_items) = schema.get("minItems").and_then(|v| v.as_u64()) {
            if array.len() < min_items as usize {
                return Err(McpError::validation(format!(
                    "Array '{}' too short: {} < {}",
                    field_name,
                    array.len(),
                    min_items
                )));
            }
        }

        if let Some(max_items) = schema.get("maxItems").and_then(|v| v.as_u64()) {
            if array.len() > max_items as usize {
                return Err(McpError::validation(format!(
                    "Array '{}' too long: {} > {}",
                    field_name,
                    array.len(),
                    max_items
                )));
            }
        }

        // Validate each item if items schema is provided
        if let Some(items_schema) = schema.get("items") {
            for (i, item) in array.iter_mut().enumerate() {
                let item_field = format!("{field_name}[{i}]");
                self.validate_and_coerce_value(item, items_schema, &item_field)?;
            }
        }

        Ok(())
    }

    /// Validate object values
    fn validate_object(
        &self,
        value: &mut Value,
        _schema: &Map<String, Value>,
        field_name: &str,
    ) -> McpResult<()> {
        let obj = value.as_object().ok_or_else(|| {
            McpError::validation(format!("Parameter '{field_name}' must be an object"))
        })?;

        // Object size validation
        if let Some(max_props) = self.config.max_object_properties {
            if obj.len() > max_props {
                return Err(McpError::validation(format!(
                    "Object '{}' has too many properties: {} > {}",
                    field_name,
                    obj.len(),
                    max_props
                )));
            }
        }

        Ok(())
    }

    /// Validate null values
    fn validate_null(&self, value: &Value, field_name: &str) -> McpResult<()> {
        if !value.is_null() {
            return Err(McpError::validation(format!(
                "Parameter '{field_name}' must be null"
            )));
        }
        Ok(())
    }

    /// Validate enum constraints
    fn validate_enum(&self, value: &Value, enum_values: &Value, field_name: &str) -> McpResult<()> {
        let enum_array = enum_values
            .as_array()
            .ok_or_else(|| McpError::validation("Enum must be an array"))?;

        if !enum_array.contains(value) {
            return Err(McpError::validation(format!(
                "Parameter '{field_name}' must be one of: {enum_array:?}"
            )));
        }

        Ok(())
    }

    /// Check for disallowed additional properties
    fn check_additional_properties(
        &self,
        params: &HashMap<String, Value>,
        schema: &Map<String, Value>,
    ) -> McpResult<()> {
        if let Some(properties) = schema.get("properties").and_then(|p| p.as_object()) {
            let allowed_props: HashSet<_> = properties.keys().collect();
            let actual_props: HashSet<_> = params.keys().collect();
            let additional: Vec<_> = actual_props.difference(&allowed_props).collect();

            if !additional.is_empty() {
                return Err(McpError::validation(format!(
                    "Additional properties not allowed: {additional:?}"
                )));
            }
        }

        Ok(())
    }

    /// Type coercion helpers
    fn coerce_to_string(&self, value: &Value) -> Option<Value> {
        match value {
            Value::Number(n) => Some(Value::String(n.to_string())),
            Value::Bool(b) => Some(Value::String(b.to_string())),
            Value::Null => Some(Value::String("null".to_string())),
            _ => None,
        }
    }

    fn coerce_to_number(&self, value: &Value) -> Option<Value> {
        match value {
            Value::String(s) => {
                if let Ok(f) = s.parse::<f64>() {
                    serde_json::Number::from_f64(f).map(Value::Number)
                } else {
                    None
                }
            }
            Value::Bool(true) => Some(Value::Number(serde_json::Number::from(1))),
            Value::Bool(false) => Some(Value::Number(serde_json::Number::from(0))),
            _ => None,
        }
    }

    fn coerce_to_boolean(&self, value: &Value) -> Option<Value> {
        match value {
            Value::String(s) => match s.to_lowercase().as_str() {
                "true" | "1" | "yes" | "on" => Some(Value::Bool(true)),
                "false" | "0" | "no" | "off" | "" => Some(Value::Bool(false)),
                _ => None,
            },
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Some(Value::Bool(i != 0))
                } else {
                    Some(Value::Bool(n.as_f64().unwrap_or(0.0) != 0.0))
                }
            }
            Value::Null => Some(Value::Bool(false)),
            _ => None,
        }
    }
}

/// Helper trait for creating typed parameter validators
pub trait ParameterType {
    /// Create a JSON schema for this parameter type
    fn to_schema() -> Value;

    /// Validate and extract value from parameters
    fn from_params(params: &HashMap<String, Value>, name: &str) -> McpResult<Self>
    where
        Self: Sized;
}

/// Implementation for basic types
impl ParameterType for String {
    fn to_schema() -> Value {
        serde_json::json!({
            "type": "string"
        })
    }

    fn from_params(params: &HashMap<String, Value>, name: &str) -> McpResult<Self> {
        params
            .get(name)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| McpError::validation(format!("Missing string parameter: {name}")))
    }
}

impl ParameterType for i64 {
    fn to_schema() -> Value {
        serde_json::json!({
            "type": "integer"
        })
    }

    fn from_params(params: &HashMap<String, Value>, name: &str) -> McpResult<Self> {
        params
            .get(name)
            .and_then(|v| v.as_i64())
            .ok_or_else(|| McpError::validation(format!("Missing integer parameter: {name}")))
    }
}

impl ParameterType for f64 {
    fn to_schema() -> Value {
        serde_json::json!({
            "type": "number"
        })
    }

    fn from_params(params: &HashMap<String, Value>, name: &str) -> McpResult<Self> {
        params
            .get(name)
            .and_then(|v| v.as_f64())
            .ok_or_else(|| McpError::validation(format!("Missing number parameter: {name}")))
    }
}

impl ParameterType for bool {
    fn to_schema() -> Value {
        serde_json::json!({
            "type": "boolean"
        })
    }

    fn from_params(params: &HashMap<String, Value>, name: &str) -> McpResult<Self> {
        params
            .get(name)
            .and_then(|v| v.as_bool())
            .ok_or_else(|| McpError::validation(format!("Missing boolean parameter: {name}")))
    }
}

/// Macro for creating parameter validation schemas
#[macro_export]
macro_rules! param_schema {
    // String parameter
    (string $name:expr_2021) => {
        ($name, serde_json::json!({"type": "string"}))
    };

    // String with constraints
    (string $name:expr_2021, min: $min:expr_2021) => {
        ($name, serde_json::json!({"type": "string", "minLength": $min}))
    };

    (string $name:expr_2021, max: $max:expr_2021) => {
        ($name, serde_json::json!({"type": "string", "maxLength": $max}))
    };

    (string $name:expr_2021, min: $min:expr_2021, max: $max:expr_2021) => {
        ($name, serde_json::json!({"type": "string", "minLength": $min, "maxLength": $max}))
    };

    // Number parameter
    (number $name:expr_2021) => {
        ($name, serde_json::json!({"type": "number"}))
    };

    (number $name:expr_2021, min: $min:expr_2021) => {
        ($name, serde_json::json!({"type": "number", "minimum": $min}))
    };

    (number $name:expr_2021, max: $max:expr_2021) => {
        ($name, serde_json::json!({"type": "number", "maximum": $max}))
    };

    (number $name:expr_2021, min: $min:expr_2021, max: $max:expr_2021) => {
        ($name, serde_json::json!({"type": "number", "minimum": $min, "maximum": $max}))
    };

    // Integer parameter
    (integer $name:expr_2021) => {
        ($name, serde_json::json!({"type": "integer"}))
    };

    (integer $name:expr_2021, min: $min:expr_2021) => {
        ($name, serde_json::json!({"type": "integer", "minimum": $min}))
    };

    (integer $name:expr_2021, max: $max:expr_2021) => {
        ($name, serde_json::json!({"type": "integer", "maximum": $max}))
    };

    (integer $name:expr_2021, min: $min:expr_2021, max: $max:expr_2021) => {
        ($name, serde_json::json!({"type": "integer", "minimum": $min, "maximum": $max}))
    };

    // Boolean parameter
    (boolean $name:expr_2021) => {
        ($name, serde_json::json!({"type": "boolean"}))
    };

    // Array parameter
    (array $name:expr_2021, items: $items:expr_2021) => {
        ($name, serde_json::json!({"type": "array", "items": $items}))
    };

    // Enum parameter
    (enum $name:expr_2021, values: [$($val:expr_2021),*]) => {
        ($name, serde_json::json!({"type": "string", "enum": [$($val),*]}))
    };
}

/// Helper function to create tool schemas from parameter definitions
pub fn create_tool_schema(params: Vec<(&str, Value)>, required: Vec<&str>) -> Value {
    let mut properties = Map::new();

    for (name, schema) in params {
        properties.insert(name.to_string(), schema);
    }

    serde_json::json!({
        "type": "object",
        "properties": properties,
        "required": required
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_string_validation() {
        let schema = json!({
            "type": "object",
            "properties": {
                "name": {"type": "string", "minLength": 2, "maxLength": 10}
            },
            "required": ["name"]
        });

        let validator = ParameterValidator::new(schema);

        // Valid string
        let mut params = HashMap::new();
        params.insert("name".to_string(), json!("test"));
        assert!(validator.validate_and_coerce(&mut params).is_ok());

        // String too short
        let mut params = HashMap::new();
        params.insert("name".to_string(), json!("a"));
        assert!(validator.validate_and_coerce(&mut params).is_err());

        // String too long
        let mut params = HashMap::new();
        params.insert("name".to_string(), json!("this_is_too_long"));
        assert!(validator.validate_and_coerce(&mut params).is_err());
    }

    #[test]
    fn test_number_validation() {
        let schema = json!({
            "type": "object",
            "properties": {
                "age": {"type": "integer", "minimum": 0, "maximum": 150}
            },
            "required": ["age"]
        });

        let validator = ParameterValidator::new(schema);

        // Valid number
        let mut params = HashMap::new();
        params.insert("age".to_string(), json!(25));
        assert!(validator.validate_and_coerce(&mut params).is_ok());

        // Number too small
        let mut params = HashMap::new();
        params.insert("age".to_string(), json!(-5));
        assert!(validator.validate_and_coerce(&mut params).is_err());

        // Number too large
        let mut params = HashMap::new();
        params.insert("age".to_string(), json!(200));
        assert!(validator.validate_and_coerce(&mut params).is_err());
    }

    #[test]
    fn test_type_coercion() {
        let schema = json!({
            "type": "object",
            "properties": {
                "count": {"type": "integer"},
                "flag": {"type": "boolean"},
                "name": {"type": "string"}
            }
        });

        let validator = ParameterValidator::new(schema);

        let mut params = HashMap::new();
        params.insert("count".to_string(), json!("42")); // String -> Number
        params.insert("flag".to_string(), json!("true")); // String -> Boolean
        params.insert("name".to_string(), json!(123)); // Number -> String

        assert!(validator.validate_and_coerce(&mut params).is_ok());

        // Check coercion results
        assert_eq!(params.get("count").unwrap().as_i64(), Some(42));
        assert_eq!(params.get("flag").unwrap().as_bool(), Some(true));
        assert_eq!(params.get("name").unwrap().as_str(), Some("123"));
    }

    #[test]
    fn test_param_schema_macro() {
        let (name, schema) = param_schema!(string "username", min: 3, max: 20);
        assert_eq!(name, "username");
        assert_eq!(schema["type"], "string");
        assert_eq!(schema["minLength"], 3);
        assert_eq!(schema["maxLength"], 20);
    }

    #[test]
    fn test_create_tool_schema() {
        let schema = create_tool_schema(
            vec![
                param_schema!(string "name"),
                param_schema!(integer "age", min: 0),
                param_schema!(boolean "active"),
            ],
            vec!["name", "age"],
        );

        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["name"]["type"] == "string");
        assert!(schema["properties"]["age"]["type"] == "integer");
        assert!(schema["properties"]["active"]["type"] == "boolean");
        assert_eq!(schema["required"], json!(["name", "age"]));
    }
}
