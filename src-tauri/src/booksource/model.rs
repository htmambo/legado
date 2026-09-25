use serde::{Deserialize, Serialize};

/// 书源元数据（与前端 `BookSourceMeta` 字段对应）。
///
/// 字段从书源 JS 文件头部注释（`// @key value`）解析而来，
/// 解析器见 `commands.rs::parse_header_meta`。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookSourceMeta {
    /// 书源唯一标识：@uuid，缺失时回退为 fileName（前端用它做 key / 健康检测索引）
    pub source_key: String,
    /// @uuid，未声明时与 source_key 相同
    pub uuid: String,
    pub file_name: String,
    pub name: String,
    /// 主 URL（第一个 @url）
    pub url: String,
    /// 全部 @url（多镜像时多条）
    pub urls: Vec<String>,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub logo: Option<String>,
    /// 多条 @description 以换行拼接
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
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub update_url: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub min_delay_ms: u64,
    #[serde(default)]
    pub require_urls: Vec<String>,
}

fn default_source_type() -> String {
    "novel".to_string()
}
