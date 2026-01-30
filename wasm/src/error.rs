//! Structured error types for WASM exports
//!
//! Provides typed errors that can be caught and handled programmatically in JavaScript.

use wasm_bindgen::prelude::*;

/// Error codes for programmatic handling in JavaScript
#[derive(Clone, Copy, Debug)]
pub enum ErrorCode {
    NotFound,
    IoError,
    ParseError,
    ValidationError,
    StorageError,
    Unknown,
}

impl ErrorCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorCode::NotFound => "NOT_FOUND",
            ErrorCode::IoError => "IO_ERROR",
            ErrorCode::ParseError => "PARSE_ERROR",
            ErrorCode::ValidationError => "VALIDATION_ERROR",
            ErrorCode::StorageError => "STORAGE_ERROR",
            ErrorCode::Unknown => "UNKNOWN",
        }
    }
}

/// Structured error for JavaScript consumption
/// 
/// Usage in JS:
/// ```js
/// try {
///     await manager.saveCheckpoint("msg");
/// } catch (error) {
///     if (error.code === "NOT_FOUND") {
///         // Handle missing file
///     }
/// }
/// ```
pub struct KamarosError {
    code: ErrorCode,
    message: String,
}

impl KamarosError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::NotFound, message)
    }

    pub fn io_error(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::IoError, message)
    }

    pub fn parse_error(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::ParseError, message)
    }

    /// Convert to JsValue for throwing from WASM functions
    pub fn to_js(self) -> JsValue {
        let obj = js_sys::Object::new();
        let _ = js_sys::Reflect::set(&obj, &"code".into(), &self.code.as_str().into());
        let _ = js_sys::Reflect::set(&obj, &"message".into(), &self.message.into());
        obj.into()
    }
}

impl From<String> for KamarosError {
    fn from(msg: String) -> Self {
        // Try to infer error type from message
        let code = if msg.to_lowercase().contains("not found") {
            ErrorCode::NotFound
        } else if msg.contains("I/O") || msg.contains("Io") {
            ErrorCode::IoError
        } else if msg.to_lowercase().contains("parse") {
            ErrorCode::ParseError
        } else {
            ErrorCode::Unknown
        };
        
        Self { code, message: msg }
    }
}

impl From<&str> for KamarosError {
    fn from(msg: &str) -> Self {
        KamarosError::from(msg.to_string())
    }
}

/// Convert KamarosError to JsValue for throwing
impl From<KamarosError> for JsValue {
    fn from(err: KamarosError) -> Self {
        // Create a JS object with code and message properties
        let obj = js_sys::Object::new();
        let _ = js_sys::Reflect::set(&obj, &"code".into(), &err.code.as_str().into());
        let _ = js_sys::Reflect::set(&obj, &"message".into(), &err.message.into());
        obj.into()
    }
}

