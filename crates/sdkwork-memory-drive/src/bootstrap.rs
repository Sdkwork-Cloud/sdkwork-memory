use std::sync::Arc;

use sdkwork_database_config::claw_database::postgres_url_with_search_path;
use sdkwork_database_config::{DatabaseConfig, DatabaseEngine as SdkDatabaseEngine};
use sdkwork_database_sqlx::{create_any_pool_from_config, PoolError};
use sdkwork_drive_config::{
    DatabaseConfig as DriveDatabaseConfig, DatabaseEngine as DriveDatabaseEngine,
};
use sdkwork_drive_workspace_service::infrastructure::sql::{
    connect_any_database_and_install_schema, install_any_schema,
};
use sdkwork_memory_spi::MemoryDriveExportUploader;
use sdkwork_utils_rust::is_blank;
use sqlx::AnyPool;

use crate::object_store::{
    build_memory_drive_object_store, load_memory_drive_storage_provider,
};
use crate::uploader::DriveUploaderMemoryExportAdapter;

const MEMORY_DRIVE_POOL_MAX_CONNECTIONS: u32 = 5;
const DEFAULT_MEMORY_DRIVE_PROVIDER_ID: &str = "sdkwork-memory-local";
const DEFAULT_MEMORY_DRIVE_BUCKET: &str = "memory";

pub async fn bootstrap_memory_drive_export_uploader_from_env(
) -> Result<Option<Arc<dyn MemoryDriveExportUploader>>, String> {
    let database_url = std::env::var("SDKWORK_MEMORY_DRIVE_DATABASE_URL")
        .or_else(|_| std::env::var("SDKWORK_DRIVE_DATABASE_URL"))
        .ok()
        .filter(|value| !is_blank(Some(value.as_str())));
    let Some(database_url) = database_url else {
        return Ok(None);
    };

    let object_store_root = std::env::var("SDKWORK_MEMORY_DRIVE_OBJECT_STORE_ROOT")
        .or_else(|_| std::env::var("SDKWORK_DRIVE_OBJECT_STORE_ROOT"))
        .unwrap_or_default();

    let pool = connect_memory_drive_pool(&database_url).await?;
    let provider =
        load_memory_drive_storage_provider(&pool, DEFAULT_MEMORY_DRIVE_PROVIDER_ID).await?;
    let object_store =
        build_memory_drive_object_store(&provider, object_store_root.as_str()).await?;
    Ok(Some(Arc::new(DriveUploaderMemoryExportAdapter::new(
        pool,
        object_store,
        DEFAULT_MEMORY_DRIVE_PROVIDER_ID.to_string(),
        DEFAULT_MEMORY_DRIVE_BUCKET.to_string(),
        "sdkwork-memory".to_string(),
    ))))
}

async fn connect_memory_drive_pool(database_url: &str) -> Result<AnyPool, String> {
    let normalized = database_url.trim();
    let engine = SdkDatabaseEngine::from_url(normalized).ok_or_else(|| {
        format!("unsupported memory drive database url: {normalized}")
    })?;
    let drive_engine = match engine {
        SdkDatabaseEngine::Sqlite => DriveDatabaseEngine::Sqlite,
        SdkDatabaseEngine::Postgres => DriveDatabaseEngine::Postgresql,
    };
    let database_config = DatabaseConfig {
        engine,
        url: if engine == SdkDatabaseEngine::Postgres {
            postgres_url_with_search_path(normalized, "SDKWORK_MEMORY")
        } else {
            normalized.to_string()
        },
        max_connections: MEMORY_DRIVE_POOL_MAX_CONNECTIONS,
        ..DatabaseConfig::default()
    };

    let pool = match drive_engine {
        DriveDatabaseEngine::Postgresql => {
            let drive_config = DriveDatabaseConfig::from_url_with_max_connections(
                database_config.url.as_str(),
                database_config.max_connections,
            )
            .map_err(|error| error.to_string())?;
            connect_any_database_and_install_schema(&drive_config)
                .await
                .map_err(|error| error.to_string())?
        }
        DriveDatabaseEngine::Sqlite => {
            let pool = create_any_pool_from_config(database_config)
                .await
                .map_err(map_pool_error)?;
            install_any_schema(&pool, drive_engine)
                .await
                .map_err(|error| error.to_string())?;
            pool
        }
    };

    seed_default_drive_storage_provider(&pool, drive_engine).await?;
    Ok(pool)
}

async fn seed_default_drive_storage_provider(
    pool: &AnyPool,
    engine: DriveDatabaseEngine,
) -> Result<(), String> {
    let exists: Option<i64> =
        sqlx::query_scalar("SELECT 1 FROM dr_drive_storage_provider WHERE id = $1")
            .bind(DEFAULT_MEMORY_DRIVE_PROVIDER_ID)
            .fetch_optional(pool)
            .await
            .map_err(|error| format!("read memory drive storage provider failed: {error}"))?;
    if exists.is_some() {
        return Ok(());
    }

    if let Some(s3_endpoint) = std::env::var("SDKWORK_MEMORY_DRIVE_S3_ENDPOINT")
        .ok()
        .filter(|value| !is_blank(Some(value.as_str())))
    {
        return seed_s3_drive_storage_provider(pool, engine, s3_endpoint.as_str()).await;
    }

    let sql = match engine {
        DriveDatabaseEngine::Sqlite => {
            "INSERT INTO dr_drive_storage_provider (
            id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, server_side_encryption_mode, default_storage_class,
            status, version, created_by, updated_by
        ) VALUES (
            $1, 'local_filesystem', $2, 'file://localhost', 'local', $2, 1, 1,
            'plain:local:local', NULL, NULL, 'active', 1, 'system', 'system'
        )"
        }
        DriveDatabaseEngine::Postgresql => {
            "INSERT INTO dr_drive_storage_provider (
            id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, server_side_encryption_mode, default_storage_class,
            status, version, created_by, updated_by
        ) VALUES (
            $1, 'local_filesystem', $2, 'file://localhost', 'local', $2, TRUE, TRUE,
            'plain:local:local', NULL, NULL, 'active', 1, 'system', 'system'
        )"
        }
    };

    sqlx::query(sql)
        .bind(DEFAULT_MEMORY_DRIVE_PROVIDER_ID)
        .bind(DEFAULT_MEMORY_DRIVE_BUCKET)
        .execute(pool)
        .await
        .map_err(|error| format!("seed memory drive storage provider failed: {error}"))?;
    Ok(())
}

async fn seed_s3_drive_storage_provider(
    pool: &AnyPool,
    engine: DriveDatabaseEngine,
    endpoint: &str,
) -> Result<(), String> {
    let region = std::env::var("SDKWORK_MEMORY_DRIVE_S3_REGION")
        .or_else(|_| std::env::var("SDKWORK_DRIVE_S3_REGION"))
        .unwrap_or_else(|_| "us-east-1".to_string());
    let bucket = std::env::var("SDKWORK_MEMORY_DRIVE_S3_BUCKET")
        .unwrap_or_else(|_| DEFAULT_MEMORY_DRIVE_BUCKET.to_string());
    let credential_ref = std::env::var("SDKWORK_MEMORY_DRIVE_S3_CREDENTIAL_REF")
        .or_else(|_| std::env::var("SDKWORK_DRIVE_S3_CREDENTIAL_REF"))
        .ok()
        .filter(|value| !is_blank(Some(value.as_str())));
    let path_style = std::env::var("SDKWORK_MEMORY_DRIVE_S3_PATH_STYLE")
        .ok()
        .map(|value| matches!(value.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes"))
        .unwrap_or(true);
    let strict_tls = std::env::var("SDKWORK_MEMORY_DRIVE_S3_STRICT_TLS")
        .ok()
        .map(|value| matches!(value.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes"))
        .unwrap_or(!endpoint.to_ascii_lowercase().starts_with("http://"));
    let credential_ref = credential_ref.unwrap_or_else(|| "env:sdkwork-drive-s3".to_string());

    let sql = match engine {
        DriveDatabaseEngine::Sqlite => {
            "INSERT INTO dr_drive_storage_provider (
            id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, server_side_encryption_mode, default_storage_class,
            status, version, created_by, updated_by
        ) VALUES (
            $1, 's3_compatible', 'Memory Export S3', $2, $3, $4, $5, $6,
            $7, NULL, NULL, 'active', 1, 'system', 'system'
        )"
        }
        DriveDatabaseEngine::Postgresql => {
            "INSERT INTO dr_drive_storage_provider (
            id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, server_side_encryption_mode, default_storage_class,
            status, version, created_by, updated_by
        ) VALUES (
            $1, 's3_compatible', 'Memory Export S3', $2, $3, $4, $5, $6,
            $7, NULL, NULL, 'active', 1, 'system', 'system'
        )"
        }
    };

    sqlx::query(sql)
        .bind(DEFAULT_MEMORY_DRIVE_PROVIDER_ID)
        .bind(endpoint)
        .bind(region)
        .bind(bucket)
        .bind(path_style)
        .bind(strict_tls)
        .bind(credential_ref)
        .execute(pool)
        .await
        .map_err(|error| format!("seed memory drive s3 storage provider failed: {error}"))?;
    Ok(())
}

fn map_pool_error(error: PoolError) -> String {
    error.to_string()
}
