use crate::curseforge::API_BASE_URL;
use crate::util::BetterJsonError;
use anyhow::{Context, bail};
use chrono::{DateTime, Utc};
use reqwest::header::{CONTENT_TYPE, HeaderValue};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use serde_repr::Deserialize_repr;
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct Mod {
    pub id: u64,
    #[serde(rename = "gameId")]
    pub game_id: u64,
    pub name: String,
    pub slug: String,
    pub links: ModLinks,
    pub summary: String,
    pub status: ModStatus,
    #[serde(rename = "downloadCount")]
    pub download_count: usize,
    #[serde(rename = "isFeatured", default = "crate::util::default_true")]
    pub is_featured: bool,
    #[serde(rename = "primaryCategoryId")]
    pub primary_category_id: u64,
    // TODO categories list
    #[serde(rename = "classId")]
    pub class_id: Option<u64>,
    pub authors: Vec<ModAuthor>,
    pub logo: ModAsset,
    pub screenshots: Vec<ModAsset>,
    #[serde(rename = "mainFileId")]
    pub main_file_id: Option<u64>,
    #[serde(rename = "latestFiles")]
    pub latest_files: Vec<File>,
    #[serde(rename = "latestFilesIndexes")]
    pub latest_files_indexes: Vec<FileIndex>,
    #[serde(rename = "latestEarlyAccessFilesIndexes")]
    pub latest_early_access_files_indexes: Vec<FileIndex>,
    #[serde(rename = "dateCreated")]
    pub date_created: DateTime<Utc>,
    #[serde(rename = "dateModified")]
    pub date_modified: Option<DateTime<Utc>>,
    #[serde(rename = "dateReleased")]
    pub date_released: Option<DateTime<Utc>>,
    #[serde(rename = "allowModDistribution", default = "crate::util::default_true")]
    pub allow_mod_distribution: bool,
    #[serde(rename = "gamePopularityRank")]
    pub game_popularity_rank: Option<usize>,
    #[serde(rename = "isAvailable", default = "crate::util::default_true")]
    pub is_available: bool,
    #[serde(default = "crate::util::default_true")]
    pub has_comments_enabled: bool,
    #[serde(rename = "thumbsUpCount")]
    pub thumbs_up_count: Option<usize>,
    pub rating: Option<f64>,
    // TODO featuredProjectTag
    // TODO socialLinks
}

#[derive(Serialize, Deserialize)]
pub struct ModLinks {
    #[serde(rename = "websiteUrl")]
    pub website_url: String,
    #[serde(rename = "wikiUrl")]
    pub wiki_url: Option<String>,
    #[serde(rename = "issuesUrl")]
    pub issues_url: Option<String>,
    #[serde(rename = "sourcesUrl")]
    pub sources_url: Option<String>,
}

#[derive(Serialize, Deserialize_repr)]
#[repr(u8)]
pub enum ModStatus {
    #[serde(rename = "new")]
    New = 1,
    #[serde(rename = "changes_required")]
    ChangesRequired = 2,
    #[serde(rename = "under_soft_review")]
    UnderSoftReview = 3,
    #[serde(rename = "approved")]
    Approved = 4,
    #[serde(rename = "rejected")]
    Rejected = 5,
    #[serde(rename = "changes_made")]
    ChangesMade = 6,
    #[serde(rename = "inactive")]
    Inactive = 7,
    #[serde(rename = "abandoned")]
    Abandoned = 8,
    #[serde(rename = "deleted")]
    Deleted = 9,
    #[serde(rename = "under_review")]
    UnderReview = 10,
}

#[derive(Serialize, Deserialize)]
pub struct ModAuthor {
    pub id: u64,
    pub name: String,
    pub url: String,
    #[serde(rename = "avatarUrl")]
    pub avatar_url: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct ModAsset {
    pub id: u64,
    #[serde(rename = "modId")]
    pub project_id: u64,
    pub title: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "thumbnailUrl")]
    pub thumbnail_url: Option<String>,
    pub url: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct File {
    pub id: u64,
    #[serde(rename = "gameId")]
    pub game_id: u64,
    #[serde(rename = "modId")]
    pub project_id: u64,
    #[serde(rename = "isAvailable", default = "crate::util::default_true")]
    pub is_available: bool,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    #[serde(rename = "fileName")]
    pub file_name: String,
    #[serde(rename = "releaseType")]
    pub release_type: FileReleaseType,
    #[serde(rename = "fileStatus")]
    pub status: FileStatus,
    pub hashes: Vec<FileHash>,
    #[serde(rename = "fileDate")]
    pub date_uploaded: DateTime<Utc>,
    #[serde(rename = "fileLength")]
    pub size: usize,
    #[serde(rename = "downloadCount")]
    pub download_count: usize,
    #[serde(rename = "fileSizeOnDisk")]
    pub size_on_disk: Option<usize>,
    #[serde(rename = "downloadUrl")]
    pub download_url: Option<String>,
    #[serde(rename = "gameVersions")]
    pub game_versions: Vec<String>,
    // #[serde(rename = "sortableGameVersions")]
    // pub sortable_game_versions: Vec<SortableGameVersion>,
    // pub dependencies: vec<FileDependency>,
    #[serde(rename = "exposeAsAlternative", default = "crate::util::default_true")]
    pub expose_as_alternative: bool,
    #[serde(rename = "parentProjectFileId")]
    pub parent_project_file_id: Option<u64>,
    #[serde(rename = "alternateFileId")]
    pub alternate_file_id: Option<u64>,
    #[serde(rename = "isServerPack", default)]
    pub is_server_pack: bool,
    #[serde(rename = "serverPackFileId")]
    pub server_pack_file_id: Option<u64>,
    #[serde(rename = "isEarlyAccessContent", default)]
    pub is_early_access_content: bool,
    #[serde(rename = "earlyAccessEndDate")]
    pub early_access_end_date: Option<DateTime<Utc>>,
    #[serde(rename = "fileFingerprint")]
    pub fingerprint: u64,
    // pub modules: Vec<FileModule>
}

#[derive(Serialize, Deserialize_repr, Clone)]
#[repr(u8)]
pub enum FileReleaseType {
    Release = 1,
    Beta = 2,
    Alpha = 3,
}

#[derive(Serialize, Deserialize_repr, Clone)]
#[repr(u8)]
pub enum FileStatus {
    #[serde(rename = "processing")]
    Processing = 1,
    #[serde(rename = "changes_required")]
    ChangesRequired = 2,
    #[serde(rename = "under_review")]
    UnderReview = 3,
    #[serde(rename = "approved")]
    Approved = 4,
    #[serde(rename = "rejected")]
    Rejected = 5,
    #[serde(rename = "malware_detected")]
    MalwareDetected = 6,
    #[serde(rename = "deleted")]
    Deleted = 7,
    #[serde(rename = "archived")]
    Archived = 8,
    #[serde(rename = "testing")]
    Testing = 9,
    #[serde(rename = "released")]
    Released = 10,
    #[serde(rename = "ready_for_review")]
    ReadyForReview = 11,
    #[serde(rename = "deprecated")]
    Deprecated = 12,
    #[serde(rename = "baking")]
    Baking = 13,
    #[serde(rename = "awaiting_publishing")]
    AwaitingPublishing = 14,
    #[serde(rename = "failed_publishing")]
    FailedPublishing = 15,
    #[serde(rename = "cooking")]
    Cooking = 16,
    #[serde(rename = "cooked")]
    Cooked = 17,
    #[serde(rename = "under_manual_review")]
    UnderManualReview = 18,
    #[serde(rename = "scanning_for_malware")]
    ScanningForMalware = 19,
    #[serde(rename = "processing_file")]
    ProcessingFile = 20,
    #[serde(rename = "pending_release")]
    PendingRelease = 21,
    #[serde(rename = "ready_for_cooking")]
    ReadyForCooking = 22,
    #[serde(rename = "post_processing")]
    PostProcessing = 23,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct FileHash {
    pub value: String,
    #[serde(rename = "algo")]
    pub algorithm: FileHashAlgorithm,
}

#[derive(Serialize, Deserialize_repr, Clone)]
#[repr(u8)]
pub enum FileHashAlgorithm {
    #[serde(rename = "sha_1")]
    SHA1 = 1,
    #[serde(rename = "md5")]
    MD5 = 2,
}

#[derive(Serialize, Deserialize)]
pub struct FileIndex {
    #[serde(rename = "gameVersion")]
    pub game_version: String,
    #[serde(rename = "fileId")]
    pub file_id: u64,
    #[serde(rename = "filename")]
    pub file_name: String,
    #[serde(rename = "releaseType")]
    pub release_type: FileReleaseType,
    #[serde(rename = "gameVersionTypeId")]
    pub game_version_type_id: Option<u64>,
    #[serde(rename = "modLoader")]
    pub mod_loader: Option<ModLoaderType>,
}

#[derive(Serialize, Deserialize_repr)]
#[repr(u8)]
pub enum ModLoaderType {
    #[serde(rename = "any")]
    Any = 0,
    #[serde(rename = "forge")]
    Forge = 1,
    #[serde(rename = "cauldron")]
    Cauldron = 2,
    #[serde(rename = "liteloader")]
    LiteLoader = 3,
    #[serde(rename = "fabric")]
    Fabric = 4,
    #[serde(rename = "quilt")]
    Quilt = 5,
    #[serde(rename = "neoforge")]
    NeoForge = 6,
}

#[derive(Deserialize)]
struct GetModResponse {
    data: Mod,
}

#[derive(Serialize)]
struct GetFilesRequest {
    #[serde(rename = "fileIds")]
    file_ids: Vec<u64>,
}

#[derive(Deserialize)]
struct GetFilesResponse {
    data: Vec<File>,
}

pub async fn get_mod(client: &Client, project_id: u64) -> anyhow::Result<Option<Mod>> {
    let url = format!("{API_BASE_URL}/v1/mods/{project_id}");
    let response = client.get(url.clone()).send().await.context(url.clone())?;

    if !response.status().is_success() {
        match response.status() {
            StatusCode::NOT_FOUND => return Ok(None),
            _ => bail!("Error trying to contact Curseforge API!"),
        }
    }

    let get_mod_response: GetModResponse = response.json_with_error().await?;
    Ok(Some(get_mod_response.data))
}

pub async fn get_files(client: &Client, file_ids: Vec<u64>) -> anyhow::Result<HashMap<u64, File>> {
    let url = format!("{API_BASE_URL}/v1/mods/files");

    let req = GetFilesRequest { file_ids };

    let response = client
        .post(url.clone())
        .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
        .json(&req)
        .send()
        .await
        .context(url.clone())?;

    if !response.status().is_success() {
        match response.status() {
            StatusCode::BAD_REQUEST => return Ok(HashMap::new()),
            StatusCode::NOT_FOUND => return Ok(HashMap::new()),
            _ => bail!("Error trying to contact Curseforge API!"),
        }
    }

    let get_files_response: GetFilesResponse = response.json_with_error().await?;
    Ok(get_files_response
        .data
        .iter()
        .map(|file| (file.id, file.clone()))
        .collect())
}

pub async fn get_file_info(client: &Client, file_id: u64) -> anyhow::Result<Option<(Mod, File)>> {
    let files = get_files(client, vec![file_id]).await?;
    match files.len() {
        0 => Ok(None),
        1 => {
            let file = files.get(&file_id).unwrap();
            let project_id = file.project_id;
            match get_mod(client, project_id).await? {
                None => bail!("Could not find project with id {project_id} for file {file_id}"),
                Some(project) => Ok(Some((project, file.clone()))),
            }
        }
        len => bail!("Expected 1 result file, got {len}"),
    }
}
