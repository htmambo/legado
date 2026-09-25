use serde::{Deserialize, Serialize};

/// 书源元数据（与前端 `BookSourceMeta` 字段对应）。
///
/// P0/P1 阶段只最小化实现：目录 + 元信息扫描。完整的 JS 解析/执行/搜索等功能留待后续。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookSourceMeta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,
    pub file_name: String,
    pub name: String,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub logo: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub file_size: u64,
    #[serde(default)]
    pub modified_at: i64,
    #[serde(default)]
    pub source_dir: String,
    #[serde(default = "default_source_type")]
    pub source_type: String,
}

fn default_source_type() -> String {
    "novel".to_string()
}