use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum AppError {
    Io(String),
    Serde(String),
    Enigo(String),
    Screenshot(String),
    Script(String),
    Window(String),
    Image(String),
    Custom(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Io(msg) => write!(f, "IO错误: {msg}"),
            AppError::Serde(msg) => write!(f, "序列化错误: {msg}"),
            AppError::Enigo(msg) => write!(f, "输入控制错误: {msg}"),
            AppError::Screenshot(msg) => write!(f, "截图错误: {msg}"),
            AppError::Script(msg) => write!(f, "脚本错误: {msg}"),
            AppError::Window(msg) => write!(f, "窗口错误: {msg}"),
            AppError::Image(msg) => write!(f, "图像错误: {msg}"),
            AppError::Custom(msg) => write!(f, "{msg}"),
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Io(e.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::Serde(e.to_string())
    }
}

impl From<image::ImageError> for AppError {
    fn from(e: image::ImageError) -> Self {
        AppError::Image(e.to_string())
    }
}

// AppError 已实现 Serialize，Tauri 的 InvokeError 有 blanket From<T: Serialize>
// 所以不需要手动实现 From<AppError> for InvokeError
