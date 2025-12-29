use std::{
    collections::HashMap,
    fs::File,
    io::{self, Read},
    path::{Path, PathBuf},
    sync::Arc,
};

use mf_file::{document::DocumentReader, error::FileError as MffError, REC_HDR};
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use serde::Serialize;
use tauri::Manager;
use zip::read::ZipArchive;

const MFF_MAGIC: &[u8; 8] = b"MFFILE01";
const MFF_PREVIEW_LIMIT: usize = 64 * 1024;
const ZIP_PREVIEW_LIMIT: u64 = 512 * 1024;

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

static DOCUMENT_CACHE: Lazy<Mutex<HashMap<String, Arc<DocumentReader>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

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
    preview_json: Option<String>,
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
    preview_json: Option<String>,
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

    let descriptor = match kind {
        FileKind::Mff => inspect_mff(&path)
            .map(FileDescriptor::Mff)
            .map_err(|e| e.to_string()),
        FileKind::Zip => inspect_zip(&path)
            .map(FileDescriptor::Zip)
            .map_err(|e| e.to_string()),
    };

    if descriptor.is_ok() && matches!(kind, FileKind::Mff) {
        let mut cache = DOCUMENT_CACHE.lock();
        cache.insert(
            path_to_string(&path),
            Arc::new(DocumentReader::open(&path).map_err(|e| e.to_string())?),
        );
    }

    descriptor
}

#[tauri::command]
fn load_mff_segment(
    path: &str,
    index: usize,
) -> Result<MffSegment, String> {
    let path = PathBuf::from(path);
    if !path.exists() {
        return Err("文件不存在".to_string());
    }
    let key = path_to_string(&path);
    let reader = get_or_open_reader(&path, &key).map_err(|e| e.to_string())?;
    read_mff_segment_from_reader(&reader, index).map_err(|e| e.to_string())
}

fn get_or_open_reader(
    path: &Path,
    key: &str,
) -> Result<Arc<DocumentReader>, InspectError> {
    if let Some(reader) = DOCUMENT_CACHE.lock().get(key).cloned() {
        return Ok(reader);
    }
    let reader = Arc::new(DocumentReader::open(path)?);
    DOCUMENT_CACHE.lock().insert(key.to_string(), Arc::clone(&reader));
    Ok(reader)
}

fn inspect_mff(path: &Path) -> Result<MffSummary, InspectError> {
    let path_str = path_to_string(path);
    let reader = get_or_open_reader(path, &path_str)?;
    let dir = reader.directory();
    let metadata = std::fs::metadata(path)?;

    let mut segments = Vec::with_capacity(dir.entries.len());
    for (index, entry) in dir.entries.iter().enumerate() {
        let payload_len = entry.length.saturating_sub(REC_HDR as u64);
        segments.push(MffSegment {
            index,
            kind: entry.kind.0.clone(),
            offset: entry.offset,
            record_length: entry.length,
            payload_length: payload_len,
            crc32: entry.crc32,
            preview_json: None,
        });
    }

    Ok(MffSummary {
        path: path_str,
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

fn read_mff_segment_from_reader(
    reader: &DocumentReader,
    index: usize,
) -> Result<MffSegment, InspectError> {
    let dir = reader.directory();
    let entry = dir.entries.get(index).ok_or_else(|| {
        InspectError::Unsupported(format!("段索引 {index} 越界"))
    })?;
    let payload_len = entry.length.saturating_sub(REC_HDR as u64);
    let preview_json = if payload_len <= MFF_PREVIEW_LIMIT as u64 {
        let mut payload: Option<Vec<u8>> = None;
        reader.read_segments(entry.kind.clone(), |i, bytes| {
            if i == index {
                payload = Some(bytes.to_vec());
            }
            Ok(())
        })?;
        payload.as_deref().and_then(decode_json_preview)
    } else {
        None
    };
    Ok(MffSegment {
        index,
        kind: entry.kind.0.clone(),
        offset: entry.offset,
        record_length: entry.length,
        payload_length: payload_len,
        crc32: entry.crc32,
        preview_json,
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
        let mut entry = archive.by_index(index)?;
        let size = entry.size();
        let compressed_size = entry.compressed_size();
        total_uncompressed += size;
        total_compressed += compressed_size;

        let raw_name = entry.name().to_string();
        let display_name = raw_name
            .trim_end_matches('/')
            .rsplit_once('/')
            .map(|(_, name)| name)
            .unwrap_or_else(|| raw_name.trim_end_matches('/'))
            .to_string();

        let modified = format_zip_modified(entry.last_modified());
        let preview_json = if !entry.is_dir()
            && size <= ZIP_PREVIEW_LIMIT
            && size <= (usize::MAX as u64)
        {
            let mut buf = Vec::with_capacity(size as usize);
            entry.read_to_end(&mut buf)?;
            decode_json_preview(&buf)
        } else {
            None
        };

        entries.push(ZipEntryInfo {
            index,
            path: raw_name,
            name: display_name,
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
            preview_json,
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

fn decode_json_preview(bytes: &[u8]) -> Option<String> {
    fn render_json(bytes: &[u8]) -> Option<String> {
        let text = std::str::from_utf8(bytes).ok()?;
        let value: serde_json::Value = serde_json::from_str(text).ok()?;
        serde_json::to_string_pretty(&value).ok()
    }

    if let Some(json) = render_json(bytes) {
        return Some(json);
    }

    if let Ok(decoded) = zstd::stream::decode_all(bytes) {
        if let Some(json) = render_json(&decoded) {
            return Some(json);
        }
    }

    None
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            inspect_file,
            load_mff_segment
        ])
        .setup(move |app| {
            #[cfg(debug_assertions)] //仅在调试时自动打开开发者工具
            {
                if let Some(window) = app.get_webview_window("main") {
                    window.open_devtools();
                }
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
