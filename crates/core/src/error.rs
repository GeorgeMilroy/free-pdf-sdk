//! Typed RPC errors: every failure crosses the protocol as `{ code, message }`.

use serde::Serialize;

/// A machine-readable error code. The wrapper adds `TIMEOUT`, `DESTROYED` and `FETCH_FAILED`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    /// The request is malformed: a missing or mistyped parameter, or a value out of range.
    BadRequest,
    /// No such method.
    UnknownMethod,
    /// The message speaks another protocol version.
    UnsupportedProtocol,
    /// The viewer has not started yet.
    NotReady,
    /// No document is open.
    NoDocument,
    /// The document could not be opened.
    LoadFailed,
    /// The document needs a password to open (a user password).
    NeedsPassword,
    /// The password is wrong.
    WrongPassword,
    /// The document's security settings don't allow this (unlock with the owner password).
    NotPermitted,
    /// The viewer is in view mode, or the document is read-only.
    ReadOnly,
    /// The XFDF could not be read.
    XfdfInvalid,
    /// A value is larger than the protocol allows.
    TooLarge,
    /// Another request is still running (one load at a time).
    Busy,
    /// Something failed inside the viewer.
    Internal,
}

impl ErrorCode {
    /// The code as it appears on the wire.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::BadRequest => "BAD_REQUEST",
            Self::UnknownMethod => "UNKNOWN_METHOD",
            Self::UnsupportedProtocol => "UNSUPPORTED_PROTOCOL",
            Self::NotReady => "NOT_READY",
            Self::NoDocument => "NO_DOCUMENT",
            Self::LoadFailed => "LOAD_FAILED",
            Self::NeedsPassword => "NEEDS_PASSWORD",
            Self::WrongPassword => "WRONG_PASSWORD",
            Self::NotPermitted => "NOT_PERMITTED",
            Self::ReadOnly => "READ_ONLY",
            Self::XfdfInvalid => "XFDF_INVALID",
            Self::TooLarge => "TOO_LARGE",
            Self::Busy => "BUSY",
            Self::Internal => "INTERNAL",
        }
    }
}

/// An error returned to the host.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RpcError {
    pub code: ErrorCode,
    pub message: String,
}

impl RpcError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self { code, message: message.into() }
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::BadRequest, message)
    }

    pub fn no_document() -> Self {
        Self::new(ErrorCode::NoDocument, "no document is open")
    }

    pub fn read_only() -> Self {
        Self::new(ErrorCode::ReadOnly, "the viewer is in view mode")
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Internal, message)
    }
}

impl std::fmt::Display for RpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code.as_str(), self.message)
    }
}

impl std::error::Error for RpcError {}

/// The RPC error for an engine edit error.
pub fn from_edit_error(e: &pdfcraft_engine::EditError) -> RpcError {
    use pdfcraft_engine::EditError as E;
    let code = match e {
        E::NoDocument => ErrorCode::NoDocument,
        E::ReadOnly(_) => ErrorCode::ReadOnly,
        E::NotPermitted(_) => ErrorCode::NotPermitted,
        E::WrongPassword => ErrorCode::WrongPassword,
        E::Data(_) => ErrorCode::XfdfInvalid,
        _ => ErrorCode::Internal,
    };
    RpcError::new(code, e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_serialize_as_screaming_snake_case() {
        for code in [ErrorCode::BadRequest, ErrorCode::UnsupportedProtocol, ErrorCode::XfdfInvalid, ErrorCode::NotPermitted] {
            let json = serde_json::to_string(&code).unwrap_or_default();
            assert_eq!(json, format!("\"{}\"", code.as_str()));
        }
        let e = RpcError::new(ErrorCode::WrongPassword, "nope");
        assert_eq!(serde_json::to_value(&e).ok(), Some(serde_json::json!({ "code": "WRONG_PASSWORD", "message": "nope" })));
    }
}
