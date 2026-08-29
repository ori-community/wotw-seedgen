use crate::{Result, Seed, FORMAT_VERSION};
use std::{
    io::{Cursor, Seek, Write},
    sync::LazyLock,
};
use wotw_seedgen_data::env_or;
use zip::{write::FileOptions, CompressionMethod, ZipWriter};

/// Zstd compression level up to 22
///
/// Some candidates from testing on It's Dangerous to go Alone:
/// - level 22 takes ~1.53s
/// - level 19 takes ~0.60s but adds ~0.8% assembly size
/// - level 15 takes ~0.09s but adds ~8.3% assembly size
/// - level 9 takes ~0.017s but adds ~15% assembly size
/// - level 4 takes ~0.004s but adds ~47% assembly size
static WOTWS_COMPRESSION_LEVEL: LazyLock<i64> =
    LazyLock::new(|| env_or("WOTWS_COMPRESSION_LEVEL", 9));

impl Seed {
    pub fn package<W: Write + Seek>(&self, obj: &mut W) -> Result<()> {
        let mut package = Package::new(obj)?;

        package.append_compressed("preload.json", serde_json::to_vec(&self.preload)?)?;
        package.append_compressed("assembly.json", serde_json::to_vec(&self.assembly)?)?;

        if let Some(seedgen_info) = &self.seedgen_info {
            package.append_compressed("seedgen_info.json", serde_json::to_vec(seedgen_info)?)?;
        }

        for (path, data) in &self.assets {
            package.append(format!("assets/{path}"), data)?;
        }

        package.finish()?;
        Ok(())
    }

    pub fn package_into_bytes(&self) -> Vec<u8> {
        let mut bytes = Cursor::new(Vec::new());
        // Write into bytes shouldn't fail
        self.package(&mut bytes).unwrap();
        bytes.into_inner()
    }
}

struct Package<'k, W: Write + Seek> {
    zip: ZipWriter<W>,
    options: FileOptions<'k, ()>,
}

impl<W: Write + Seek> Package<'_, W> {
    fn new(obj: W) -> Result<Self> {
        let zip = ZipWriter::new(obj);
        let options = FileOptions::default()
            .compression_method(CompressionMethod::Zstd)
            .compression_level(Some(*WOTWS_COMPRESSION_LEVEL));

        let mut package = Self { zip, options };
        package.append("format_version.txt", FORMAT_VERSION)?;

        Ok(package)
    }

    fn append<S: Into<String>, D: AsRef<[u8]>>(&mut self, name: S, data: D) -> Result<()> {
        self.append_with(name.into(), data.as_ref(), FileOptions::default())
    }

    fn append_compressed<S: Into<String>, D: AsRef<[u8]>>(
        &mut self,
        name: S,
        data: D,
    ) -> Result<()> {
        self.append_with(name.into(), data.as_ref(), self.options)
    }

    fn append_with(&mut self, name: String, data: &[u8], options: FileOptions<()>) -> Result<()> {
        self.zip.start_file(name, options)?;
        self.zip.write_all(data)?;
        Ok(())
    }

    fn finish(self) -> Result<()> {
        self.zip.finish()?;
        Ok(())
    }
}
