use serde::Serialize;
use std::io;

/// 统一的 Tauri 命令错误类型
///
/// 前端 invoke 失败时拿到 `{ code, message }`，便于按 code 分支处理。
#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error("IO 错误: {0}")]
    Io(#[from] io::Error),

    #[error("JSON 解析错误: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("未找到: {0}")]
    NotFound(String),

    #[error("参数无效: {0}")]
    Invalid(String),

    #[error("{0}")]
    Other(String),
}

impl CommandError {
    pub fn other(msg: impl Into<String>) -> Self {
        Self::Other(msg.into())
    }

    pub fn invalid(msg: impl Into<String>) -> Self {
        Self::Invalid(msg.into())
    }

    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }
}

/// 前端拿到的形状：与 Naive UI / 全局 toast 期望的 `{ code, message }` 对齐
impl Serialize for CommandError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let code = match self {
            CommandError::Io(_) => "IO_ERROR",
            CommandError::Serde(_) => "SERDE_ERROR",
            CommandError::NotFound(_) => "NOT_FOUND",
            CommandError::Invalid(_) => "INVALID_ARG",
            CommandError::Other(_) => "OTHER",
        };
        let mut s = serializer.serialize_struct("CommandError", 2)?;
        s.serialize_field("code", code)?;
        s.serialize_field("message", &self.to_string())?;
        s.end()
    }
}

/// 业务结果类型别名
pub type CommandResult<T> = Result<T, CommandError>;