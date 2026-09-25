use serde::{Deserialize, Serialize};

/// 前端扩展元数据（对应前端 `ExtensionMeta`）。
///
/// P0 阶段只返回文件级元信息；`namespace / matchPatterns / grants / runAt / category`
/// 等头部字段前端用正则自己解析，所以这里就返回前端能直接使用的最小子集。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionMeta {
    pub file_name: String,
    pub name: String,
    pub namespace: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub match_patterns: Vec<String>,
    pub grants: Vec<String>,
    pub run_at: String,
    pub category: String,
    pub enabled: bool,
    pub file_size: u64,
    pub modified_at: i64,
}