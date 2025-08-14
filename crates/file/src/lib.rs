pub mod error;
pub mod record;
pub mod document;
pub mod history;
pub mod zipdoc;
pub use error::{FileError, Result};
pub use record::{Writer, Reader, Iter, HEADER_LEN, REC_HDR};
pub use document::{DocumentWriter, DocumentReader, SegmentType, Directory, SegmentEntry};
pub use history::{TypeWrapper, encode_history_frames, decode_history_frames};
pub use zipdoc::{ZipDocumentWriter, ZipDocumentReader};

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn roundtrip() -> Result<()> {
        let dir = tempdir().unwrap();
        let path = dir.path().join("data.mff");

        let mut w = Writer::create(&path, 64 * 1024 * 1024)?;
        let off1 = w.append(b"hello")?;
        let off2 = w.append(b"world")?;
        let big = vec![42u8; 128 * 1024];
        let off3 = w.append(&big)?;
        w.flush()?;

        assert!(off2 > off1 && off3 > off2);

        let r = Reader::open(&path)?;
        assert_eq!(r.get_at(off1)?, b"hello");
        assert_eq!(r.get_at(off2)?, b"world");
        assert_eq!(r.get_at(off3)?, &big[..]);
        assert_eq!(r.iter().count(), 3);

        drop(w);
        let mut w2 = Writer::create(&path, 64 * 1024 * 1024)?;
        let off4 = w2.append(b"!")?;
        w2.flush()?;

        let r2 = Reader::open(&path)?;
        assert_eq!(r2.get_at(off4)?, b"!");
        Ok(())
    }
}

// moved implementations to modules: document.rs, history.rs, zipdoc.rs



