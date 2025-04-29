use serde::{Serialize, Serializer};

pub type AiterResult<T> = Result<T, AiterError>;

#[derive(Debug, thiserror::Error)]
pub enum AiterError {
    #[error("[Csv Error] {0}")]
    CsvError(#[from] csv::Error),

    #[error("[Concurrent Error] {0}")]
    ConcurrentError(#[from] ::tokio::task::JoinError),

    #[error("[Database Error] {0}")]
    DatabaseError(#[from] libsql::Error),

    #[error("[Docx Error] {0}")]
    DocxError(#[from] docx_rs::ReaderError),

    #[error("[Enum Error] {0}")]
    EnumError(#[from] ::strum::ParseError),

    #[error("[Epub Error] {0}")]
    EpubError(#[from] ::epub::doc::DocError),

    #[error("[Event Error] {0}")]
    EventError(String),

    #[error("[Hash Error] {0}")]
    HashError(String),

    #[error("[HTTP Request Error] {0}")]
    HttpRequestError(#[from] reqwest::Error),

    #[error("[HTTP Status Error] {0}")]
    HttpStatusError(String),

    #[error("[Invalid] {0}")]
    Invalid(String),

    #[error("[Interrupted] {0}")]
    Interrupted(String),

    #[error("[IO Error] {0}")]
    IoError(#[from] std::io::Error),

    #[error("[MCP Error] {0}")]
    McpError(#[from] rmcp::service::ServiceError),

    #[error("[None]")]
    None,

    #[error("[Not Exists] {0}")]
    NotExists(String),

    #[error("[Pdf Error] {0}")]
    PdfError(#[from] lopdf::Error),

    #[error("[Serde Error] {0}")]
    SerdeError(#[from] serde_json::Error),

    #[error("[Signature Error] {0}")]
    SignatureError(String),

    #[error("[Timeout]")]
    Timeout,

    #[error("[URL Parse Error] {0}")]
    UrlParseError(#[from] url::ParseError),

    #[error("[Unsupported] {0}")]
    Unsupported(String),

    #[error("[Xlsx Error] {0}")]
    XlsxError(#[from] calamine::Error),
}

impl<T> From<tokio::sync::mpsc::error::SendError<T>> for AiterError {
    fn from(err: tokio::sync::mpsc::error::SendError<T>) -> Self {
        AiterError::EventError(err.to_string())
    }
}

impl From<tokio::sync::oneshot::error::RecvError> for AiterError {
    fn from(err: tokio::sync::oneshot::error::RecvError) -> Self {
        AiterError::EventError(err.to_string())
    }
}

impl Serialize for AiterError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.to_string().serialize(serializer)
    }
}
