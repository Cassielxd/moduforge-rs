use std::{
    fs::File,
    io::{self, Read},
    path::{Path, PathBuf},
};

use mf_file::{document::DocumentReader, error::FileError as MffError, REC_HDR};
use serde::Serialize;
use zip::read::ZipArchive;

const MFF_MAGIC: &[u8; 8] = b"MFFILE01";

#[derive(Debug)]
enum InspectError {
    Io(io::Error),
    File(MffError),
    Zip(zip::result::ZipError),
    Unsupported(String),
}

impl From<io::Error> for InspectError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<MffError> for InspectError {
    fn from(value: MffError) -> Self {
        Self::File(value)
    }
}

impl From<zip::result::ZipError> for InspectError {
    fn from(value: zip::result::ZipError) -> Self {
        Self::Zip(value)
    }
}

impl std::fmt::Display for InspectError {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        match self {
            InspectError::Io(e) => write!(f, "IO 错误: {e}"),
            InspectError::File(e) => write!(f, "MFF 解析失败: {e}"),
            InspectError::Zip(e) => write!(f, "ZIP 解析失败: {e}"),
            InspectError::Unsupported(msg) => write!(f, "{msg}"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum FileKind {
    Mff,
    Zip,
}

#[derive(Serialize)]
#[serde(tag = "kind", content = "data", rename_all = "lowercase")]
enum FileDescriptor {
    Mff(MffSummary),
    Zip(ZipSummary),
}

#[derive(Serialize)]
struct MffSegment {
    index: usize,
    kind: String,
    offset: u64,
    record_length: u64,
    payload_length: u64,
    crc32: u32,
}

#[derive(Serialize)]
struct MffSummary {
    path: String,
    file_name: String,
    file_size: u64,
    logical_len: u64,
    segment_count: usize,
    directory_flags: u32,
    file_hash: String,
    segments: Vec<MffSegment>,
}

#[derive(Serialize)]
struct ZipEntryInfo {
    index: usize,
    path: String,
    name: String,
    is_dir: bool,
    size: u64,
    compressed_size: u64,
    compression: String,
    compression_ratio: f64,
    crc32: u32,
    modified: Option<String>,
}

#[derive(Serialize)]
struct ZipSummary {
    path: String,
    file_name: String,
    file_size: u64,
    total_entries: usize,
    total_uncompressed: u64,
    total_compressed: u64,
    entries: Vec<ZipEntryInfo>,
}

#[tauri::command]
fn inspect_file(path: &str) -> Result<FileDescriptor, String> {
    let path = PathBuf::from(path);
    if !path.exists() {
        return Err("文件不存在".to_string());
    }

    let kind = detect_kind(&path).map_err(|e| e.to_string())?;

    match kind {
        FileKind::Mff => inspect_mff(&path)
            .map(FileDescriptor::Mff)
            .map_err(|e| e.to_string()),
        FileKind::Zip => inspect_zip(&path)
            .map(FileDescriptor::Zip)
            .map_err(|e| e.to_string()),
    }
}

fn inspect_mff(path: &Path) -> Result<MffSummary, InspectError> {
    let reader = DocumentReader::open(path)?;
    let dir = reader.directory();
    let metadata = std::fs::metadata(path)?;

    let segments = dir
        .entries
        .iter()
        .enumerate()
        .map(|(index, entry)| {
            let payload_len = entry.length.saturating_sub(REC_HDR as u64);
            MffSegment {
                index,
                kind: entry.kind.0.clone(),
                offset: entry.offset,
                record_length: entry.length,
                payload_length: payload_len,
                crc32: entry.crc32,
            }
        })
        .collect();

    Ok(MffSummary {
        path: path_to_string(path),
        file_name: path
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or_default()
            .to_string(),
        file_size: metadata.len(),
        logical_len: reader.logical_len(),
        segment_count: dir.entries.len(),
        directory_flags: dir.flags,
        file_hash: to_hex(&dir.file_hash),
        segments,
    })
}

fn inspect_zip(path: &Path) -> Result<ZipSummary, InspectError> {
    let file = File::open(path)?;
    let mut archive = ZipArchive::new(file)?;
    let total_entries = archive.len();
    let metadata = std::fs::metadata(path)?;

    let mut entries = Vec::with_capacity(total_entries);
    let mut total_uncompressed = 0u64;
    let mut total_compressed = 0u64;

    for index in 0..total_entries {
        let entry = archive.by_index(index)?;
        let size = entry.size();
        let compressed_size = entry.compressed_size();
        total_uncompressed += size;
        total_compressed += compressed_size;

        let name = entry.name().to_string();
        let display_name = entry
            .name()
            .trim_end_matches('/')
            .rsplit_once('/')
            .map(|(_, name)| name)
            .unwrap_or_else(|| entry.name().trim_end_matches('/'));

        let modified = format_zip_modified(entry.last_modified());

        entries.push(ZipEntryInfo {
            index,
            path: name,
            name: display_name.to_string(),
            is_dir: entry.is_dir(),
            size,
            compressed_size,
            compression: format!("{:?}", entry.compression()),
            compression_ratio: if size > 0 {
                compressed_size as f64 / size as f64
            } else {
                1.0
            },
            crc32: entry.crc32(),
            modified,
        });
    }

    Ok(ZipSummary {
        path: path_to_string(path),
        file_name: path
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or_default()
            .to_string(),
        file_size: metadata.len(),
        total_entries,
        total_uncompressed,
        total_compressed,
        entries,
    })
}

fn detect_kind(path: &Path) -> Result<FileKind, InspectError> {
    if let Some(ext) = path.extension().and_then(|ext| ext.to_str()) {
        match ext.to_ascii_lowercase().as_str() {
            "mff" => return Ok(FileKind::Mff),
            "zip" | "ysf" => return Ok(FileKind::Zip),
            _ => {},
        }
    }

    let mut buf = [0u8; 8];
    let mut file = File::open(path)?;
    let read = file.read(&mut buf)?;
    if read >= 4 && &buf[..4] == b"PK\x03\x04" {
        return Ok(FileKind::Zip);
    }
    if read >= 8 && &buf[..8] == MFF_MAGIC {
        return Ok(FileKind::Mff);
    }
    Err(InspectError::Unsupported(
        "无法识别的文件格式，请确保是 .mff/.ysf/.zip".into(),
    ))
}

trait IntoZipDateTimeOption {
    fn into_option(self) -> Option<zip::DateTime>;
}

impl IntoZipDateTimeOption for zip::DateTime {
    fn into_option(self) -> Option<zip::DateTime> {
        Some(self)
    }
}

impl IntoZipDateTimeOption for Option<zip::DateTime> {
    fn into_option(self) -> Option<zip::DateTime> {
        self
    }
}

fn format_zip_datetime(dt: zip::DateTime) -> String {
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        dt.year(),
        dt.month(),
        dt.day(),
        dt.hour(),
        dt.minute(),
        dt.second()
    )
}

fn format_zip_modified<T>(dt: T) -> Option<String>
where
    T: IntoZipDateTimeOption,
{
    dt.into_option().map(format_zip_datetime)
}

fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect::<Vec<_>>().join("")
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![inspect_file])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
