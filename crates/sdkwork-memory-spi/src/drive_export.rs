use async_trait::async_trait;

use crate::MemorySpiResult;

#[derive(Debug, Clone)]
pub struct MemoryDriveExportUploadRequest {
    pub tenant_id: i64,
    pub organization_id: Option<i64>,
    pub user_id: Option<i64>,
    pub export_job_id: u64,
    pub format: String,
    pub drive_target_ref: String,
    pub body: Vec<u8>,
    pub content_type: String,
    pub original_file_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryDriveExportUploadResult {
    pub drive_object_ref: String,
    pub drive_node_id: String,
    pub checksum_sha256_hex: String,
}

#[async_trait]
pub trait MemoryDriveExportUploader: Send + Sync {
    async fn upload_export(
        &self,
        request: MemoryDriveExportUploadRequest,
    ) -> MemorySpiResult<MemoryDriveExportUploadResult>;
}
