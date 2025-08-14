mod writer;
mod reader;
mod snapshot;
pub mod formats;

pub use writer::ZipDocumentWriter;
pub use reader::ZipDocumentReader;
pub use snapshot::{
    SnapshotShardMeta,
    write_snapshot_shards,
    read_snapshot_shards,
    read_and_decode_snapshot_shards,
    for_each_snapshot_shard_raw,
    export_zip_with_shards,
    import_zip_with_shards,
};
// JSON/CBOR/MessagePack 专用 API 迁移到 formats 模块命名空间


