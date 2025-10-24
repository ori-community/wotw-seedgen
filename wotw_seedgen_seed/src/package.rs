use crate::{Result, Seed, FORMAT_VERSION};
use std::io::{Seek, Write};
use zip::{write::FileOptions, CompressionMethod, ZipWriter};

impl Seed {
    pub fn package<W: Write + Seek>(&self, obj: &mut W, compress: bool) -> Result<()> {
        let f = if compress {
            Package::append_compressed
        } else {
            Package::append
        };

        let mut package = Package::new(obj)?;

        package.append("preload.json", serde_json::to_vec(&self.preload)?)?;

        f(
            &mut package,
            "assembly.json",
            serde_json::to_vec(&self.assembly)?,
        )?;

        if let Some(seedgen_info) = &self.seedgen_info {
            package.append("seedgen_info.json", serde_json::to_vec(seedgen_info)?)?;
        }

        for (path, data) in &self.assets {
            package.append(format!("assets/{path}"), data)?;
        }

        package.finish()?;
        Ok(())
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
            .compression_level(Some(22));

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
