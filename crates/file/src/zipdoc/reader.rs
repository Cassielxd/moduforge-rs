use std::io::{self, Read, Seek};
use zip::ZipArchive;

// 基于 ZIP 的文档读取器
pub struct ZipDocumentReader<R: Read + Seek> {
    pub(crate) zip: ZipArchive<R>,
}

impl<R: Read + Seek> ZipDocumentReader<R> {
    // 打开读取器
    pub fn new(r: R) -> io::Result<Self> {
        Ok(Self { zip: ZipArchive::new(r)? })
    }
    // 读取指定文件完整内容
    pub fn read_all(
        &mut self,
        name: &str,
    ) -> io::Result<Vec<u8>> {
        let mut f = self.zip.by_name(name)?;
        let mut buf = Vec::with_capacity(f.size() as usize);
        std::io::copy(&mut f, &mut buf)?;
        Ok(buf)
    }
}
