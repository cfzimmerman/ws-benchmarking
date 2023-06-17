#[derive(Debug)]
/// WsError: defines the errors that may be returned from WS
pub enum WsError {
    IoError(std::io::Error),
    ClientNotFound,
    TungsteniteError(tokio_tungstenite::tungstenite::Error),
    MutexError(String),
    SerdeError(String),
    SendError(String),
    FailedToSend(Vec<String>),
}

impl From<std::io::Error> for WsError {
    fn from(err: std::io::Error) -> Self {
        WsError::IoError(err)
    }
}

impl From<tokio_tungstenite::tungstenite::Error> for WsError {
    fn from(err: tokio_tungstenite::tungstenite::Error) -> Self {
        WsError::TungsteniteError(err)
    }
}

impl<T> From<std::sync::PoisonError<T>> for WsError {
    fn from(err: std::sync::PoisonError<T>) -> Self {
        WsError::MutexError(err.to_string())
    }
}

impl From<serde_json::Error> for WsError {
    fn from(err: serde_json::Error) -> Self {
        WsError::SerdeError(err.to_string())
    }
}

impl<T> From<futures_channel::mpsc::TrySendError<T>> for WsError {
    fn from(err: futures_channel::mpsc::TrySendError<T>) -> Self {
        WsError::SendError(err.to_string())
    }
}
