use std::path::PathBuf;
use std::sync::Mutex;

use crate::bookshelf::model::ShelfBook;
use crate::errors::CommandResult;

/// 进程级共享状态
///
/// - `data_dir`: 应用数据根目录（Tauri `app_data_dir()`）
/// - `books`: 书架内存缓存；写命令先更新内存再落盘，避免每次 list 都读盘
pub struct AppState {
    pub data_dir: PathBuf,
    pub books: Mutex<Vec<ShelfBook>>,
}

impl AppState {
    /// 启动时调用：确保数据根目录存在 + 从 books.json 加载内存缓存
    pub fn load(data_dir: PathBuf) -> CommandResult<Self> {
        std::fs::create_dir_all(&data_dir)?;
        let books_path = data_dir.join("bookshelf").join("books.json");
        let books = if books_path.exists() {
            let raw = std::fs::read_to_string(&books_path)?;
            if raw.trim().is_empty() {
                Vec::new()
            } else {
                serde_json::from_str(&raw)?
            }
        } else {
            Vec::new()
        };
        Ok(Self {
            data_dir,
            books: Mutex::new(books),
        })
    }

    /// 书架根目录（`<data_dir>/bookshelf/`）
    pub fn bookshelf_dir(&self) -> PathBuf {
        self.data_dir.join("bookshelf")
    }
}