//! Example demonstrating cross-compatibility between sync and async implementations
//!
//! This example shows how files written by either the synchronous or asynchronous
//! implementations can be read by the other, maintaining full compatibility.

use mf_file::{
    DocumentWriter, DocumentReader,
    AsyncDocumentWriter, AsyncDocumentReader,
    SegmentType, Result,
};
use tokio;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Cross-Compatibility Example ===\n");

    // Example 1: Write with sync, read with async
    println!("1. Writing document with SYNC implementation...");
    let sync_file = "example_sync.mfd";
    {
        let mut writer = DocumentWriter::begin(sync_file)?;
        writer.add_segment(SegmentType("metadata".to_string()), b"version=1.0")?;
        writer.add_segment(SegmentType("content".to_string()), b"Hello from sync!")?;
        writer.add_segment(SegmentType("binary".to_string()), &vec![0xFF; 1000])?;
        writer.finalize()?;
        println!("   ✓ Sync document written to {}", sync_file);
    }

    println!("\n2. Reading sync-generated document with ASYNC implementation...");
    {
        let reader = AsyncDocumentReader::open(sync_file).await?;

        let metadata = reader.get_segment(&SegmentType("metadata".to_string())).await?;
        println!("   Metadata: {}", String::from_utf8_lossy(&metadata.unwrap()));

        let content = reader.get_segment(&SegmentType("content".to_string())).await?;
        println!("   Content: {}", String::from_utf8_lossy(&content.unwrap()));

        let binary = reader.get_segment(&SegmentType("binary".to_string())).await?;
        println!("   Binary data size: {} bytes", binary.unwrap().len());

        println!("   ✓ Successfully read sync document with async reader");
    }

    // Example 2: Write with async, read with sync
    println!("\n3. Writing document with ASYNC implementation...");
    let async_file = "example_async.mfd";
    {
        let writer = AsyncDocumentWriter::begin_standard(async_file).await?;
        writer.add_segment(SegmentType("header".to_string()), b"async-doc-v2".to_vec()).await?;
        writer.add_segment(SegmentType("data".to_string()), b"Data from async!".to_vec()).await?;

        // Large data to test compression
        let large_data = vec![42u8; 50_000];
        writer.add_segment(SegmentType("large".to_string()), large_data).await?;

        writer.finalize().await?;
        println!("   ✓ Async document written to {}", async_file);
    }

    println!("\n4. Reading async-generated document with SYNC implementation...");
    {
        let reader = DocumentReader::open(async_file)?;

        // Read all header segments
        reader.read_segments(SegmentType("header".to_string()), |_index, data| {
            println!("   Header: {}", String::from_utf8_lossy(data));
            Ok(())
        })?;

        // Read all data segments
        reader.read_segments(SegmentType("data".to_string()), |_index, data| {
            println!("   Data: {}", String::from_utf8_lossy(data));
            Ok(())
        })?;

        // Check large segment
        let large_payload = reader.segment_payload(2)?;
        println!("   Large data size: {} bytes (compressed in file)", large_payload.len());

        println!("   ✓ Successfully read async document with sync reader");
    }

    // Example 3: Parallel compression mode (async only, but still readable by sync)
    println!("\n5. Writing document with ASYNC parallel compression...");
    let parallel_file = "example_parallel.mfd";
    {
        let writer = AsyncDocumentWriter::begin(parallel_file).await?;

        // Batch write multiple segments in parallel
        let segments = vec![
            (SegmentType("seg1".to_string()), b"First segment".to_vec()),
            (SegmentType("seg2".to_string()), b"Second segment".to_vec()),
            (SegmentType("seg3".to_string()), b"Third segment".to_vec()),
        ];

        writer.add_segments_batch(segments).await?;

        // Very large segment that benefits from parallel compression
        let huge_data = vec![99u8; 500_000];
        writer.add_segment(SegmentType("huge".to_string()), huge_data).await?;

        writer.finalize().await?;
        println!("   ✓ Parallel-compressed document written to {}", parallel_file);
    }

    println!("\n6. Reading parallel-compressed document with SYNC implementation...");
    {
        let reader = DocumentReader::open(parallel_file)?;

        println!("   Total segments: {}", reader.segments().len());

        // Read each segment type
        for (i, segment) in reader.segments().iter().enumerate() {
            println!("   Segment {}: type='{}', size={} bytes",
                     i, segment.kind.0, segment.length);
        }

        // Verify we can read the huge segment
        reader.read_segments(SegmentType("huge".to_string()), |_index, data| {
            println!("   Huge segment decompressed size: {} bytes", data.len());
            assert_eq!(data[0], 99);
            assert_eq!(data[499_999], 99);
            Ok(())
        })?;

        println!("   ✓ Successfully read parallel-compressed document with sync reader");
    }

    // Clean up example files
    println!("\n7. Cleaning up example files...");
    std::fs::remove_file(sync_file).ok();
    std::fs::remove_file(async_file).ok();
    std::fs::remove_file(parallel_file).ok();
    println!("   ✓ Example files removed");

    println!("\n=== Cross-Compatibility Verified! ===");
    println!("Both sync and async implementations can read each other's files seamlessly.");

    Ok(())
}