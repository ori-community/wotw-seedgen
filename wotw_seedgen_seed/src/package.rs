use crate::{
    assembly::Assembly, plando_attributes::PlandoAttributes, Result, Seed, SeedgenInfo,
    FORMAT_VERSION,
};
use serde::de::DeserializeOwned;
use std::{
    io::{Cursor, Read, Seek, Write},
    sync::LazyLock,
};
use wotw_seedgen_data::env_or;
use zip::{read::ZipFile, write::FileOptions, CompressionMethod, ZipArchive, ZipWriter};

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

const PRELOAD_PATH: &str = "preload.json";
const ASSEMBLY_PATH: &str = "assembly.json";
const SEEDGEN_INFO_PATH: &str = "seedgen_info.json";
const PLANDO_ATTRIBUTES_PATH: &str = "plando_attributes.json";

impl Seed {
    pub fn package<W: Write + Seek>(&self, obj: &mut W) -> Result<()> {
        let mut package = Package::new(obj)?;

        package.append_compressed(PRELOAD_PATH, serde_json::to_vec(&self.preload)?)?;
        package.append_compressed(ASSEMBLY_PATH, serde_json::to_vec(&self.assembly)?)?;

        if let Some(seedgen_info) = &self.seedgen_info {
            package.append_compressed(SEEDGEN_INFO_PATH, serde_json::to_vec(seedgen_info)?)?;
        }

        if let Some(plando_attributes) = &self.plando_attributes {
            package.append_compressed(
                PLANDO_ATTRIBUTES_PATH,
                serde_json::to_vec(plando_attributes)?,
            )?;
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

pub struct SeedReader<R> {
    inner: ZipArchive<R>,
}

impl<R: Read + Seek> SeedReader<R> {
    pub fn new(reader: R) -> Result<Self> {
        Ok(Self {
            inner: ZipArchive::new(reader)?,
        })
    }

    pub fn read_assembly(&mut self) -> Result<Assembly> {
        self.json_by_name(ASSEMBLY_PATH)
    }

    pub fn read_seedgen_info(&mut self) -> Result<SeedgenInfo> {
        self.json_by_name(SEEDGEN_INFO_PATH)
    }

    pub fn read_plando_attributes(&mut self) -> Result<PlandoAttributes> {
        self.json_by_name(PLANDO_ATTRIBUTES_PATH)
    }

    fn json_by_name<T: DeserializeOwned>(&mut self, name: &str) -> Result<T> {
        Ok(serde_json::from_reader(self.by_name(name)?)?)
    }

    fn by_name<'s>(&'s mut self, name: &str) -> Result<ZipFile<'s, R>> {
        Ok(self
            .inner
            .by_name(name)
            .map_err(|err| format!("failed to read \"{name}\" from seed: {err}"))?)
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
