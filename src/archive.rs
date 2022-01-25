use std::path::Path;

#[derive(Debug)]
pub enum ArchiveFormat {
    Z,
    Zip,
    Gzip,
    Bzip2,
    Lz,
    Xz,
    Lzma,
    P7z,
    Tar,
    TarZ,
    TarGzip,
    TarBzip2,
    TarLz,
    TarXz,
    TarLzma,
    Tar7z,
    TarZstd,
    Rar,
    Zstd,
}

impl ArchiveFormat {
    pub fn from_filename<S: AsRef<str>>(filename: S) -> Result<ArchiveFormat, &'static str> {
        let filename = filename.as_ref();

        if filename.ends_with("tar.z") {
            return Ok(ArchiveFormat::TarZ);
        } else if filename.ends_with(".tar.gz") || filename.ends_with(".tgz") {
            return Ok(ArchiveFormat::TarGzip);
        } else if filename.ends_with(".tar.bz2") || filename.ends_with(".tbz2") {
            return Ok(ArchiveFormat::TarBzip2);
        } else if filename.ends_with(".tar.lz") {
            return Ok(ArchiveFormat::TarLz);
        } else if filename.ends_with(".tar.xz") || filename.ends_with(".txz") {
            return Ok(ArchiveFormat::TarXz);
        } else if filename.ends_with(".tar.lzma") || filename.ends_with(".tlz") {
            return Ok(ArchiveFormat::TarLzma);
        } else if filename.ends_with(".tar.7z")
            || filename.ends_with(".tar.7z.001")
            || filename.ends_with(".t7z")
        {
            return Ok(ArchiveFormat::Tar7z);
        } else if filename.ends_with(".tar.zst") {
            return Ok(ArchiveFormat::TarZstd);
        } else if filename.ends_with(".tar") {
            return Ok(ArchiveFormat::Tar);
        } else if filename.ends_with(".z") {
            return Ok(ArchiveFormat::Z);
        } else if filename.ends_with(".zip") {
            return Ok(ArchiveFormat::Zip);
        } else if filename.ends_with(".gz") {
            return Ok(ArchiveFormat::Gzip);
        } else if filename.ends_with(".bz2") {
            return Ok(ArchiveFormat::Bzip2);
        } else if filename.ends_with(".lz") {
            return Ok(ArchiveFormat::Lz);
        } else if filename.ends_with(".xz") {
            return Ok(ArchiveFormat::Xz);
        } else if filename.ends_with(".lzma") {
            return Ok(ArchiveFormat::Lzma);
        } else if filename.ends_with(".7z") || filename.ends_with(".7z.001") {
            return Ok(ArchiveFormat::P7z);
        } else if filename.ends_with(".rar") {
            return Ok(ArchiveFormat::Rar);
        } else if filename.ends_with(".zst") {
            return Ok(ArchiveFormat::Zstd);
        }

        Err("Unknown archive format.")
    }
}
