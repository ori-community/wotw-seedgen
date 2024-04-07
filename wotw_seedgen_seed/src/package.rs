use crate::{Result, Seed, FORMAT_VERSION};
use std::{io::Write, path::Path};

impl Seed {
    pub fn package<W: Write>(&self, obj: &mut W) -> Result<()> {
        let mut package = Package::new(obj)?;
        package.append("preload.json", serde_json::to_vec(&self.preload)?)?;
        package.append("assembly.json", serde_json::to_vec(&self.assembly)?)?;
        for (path, data) in &self.assets {
            package.append(path, data)?;
        }
        package.finish()?;
        Ok(())
    }
}

struct Package<W: Write> {
    builder: tar::Builder<xz2::write::XzEncoder<W>>,
    header: tar::Header,
}

impl<W: Write> Package<W> {
    fn new(obj: W) -> Result<Self> {
        // TODO choose compression
        let builder = tar::Builder::new(xz2::write::XzEncoder::new(obj, 9));
        // let mut builder = tar::Builder::new(brotli::CompressorWriter::new(w, 4096, 11, 22));
        // let mut builder = tar::Builder::new(w);
        let header = base_header();
        let mut package = Self { builder, header };
        package.append("format_version.txt", FORMAT_VERSION)?;
        Ok(package)
    }

    fn append<P: AsRef<Path>, D: AsRef<[u8]>>(&mut self, path: P, data: D) -> Result<()> {
        let data = data.as_ref();
        self.header.set_path(path)?;
        self.header.set_size(data.len().try_into()?);
        self.header.set_cksum();
        self.builder.append(&self.header, data)?;
        Ok(())
    }

    fn finish(self) -> Result<()> {
        self.builder.into_inner()?.finish()?.flush()?;
        Ok(())
    }
}

fn base_header() -> tar::Header {
    let mut header = tar::Header::new_old();
    header.set_mode(0o644);
    header
}
