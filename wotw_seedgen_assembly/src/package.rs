use crate::{compile_intermediate_output, Result, SeedWorld, VERSION};
use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};
use wotw_seedgen_seed_language::output::CompilerOutput;

pub struct Package {
    builder: tar::Builder<xz2::write::XzEncoder<File>>,
    header: tar::Header,
}

impl Package {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::create(path)?;
        // TODO choose compression
        let builder = tar::Builder::new(xz2::write::XzEncoder::new(file, 9));
        // let mut builder = tar::Builder::new(brotli::CompressorWriter::new(w, 4096, 11, 22));
        // let mut builder = tar::Builder::new(w);
        let header = base_header();
        let mut package = Self { builder, header };
        package.set_header_and_append("format_version", VERSION)?;
        Ok(package)
    }

    pub fn add_from_intermediate_output(
        &mut self,
        output: CompilerOutput,
        pretty: bool,
    ) -> Result<()> {
        let (seed_world, icons) = compile_intermediate_output(output);
        self.add_seed(&seed_world, pretty)?;
        for (name, icon) in icons {
            let mut path = PathBuf::from("assets");
            path.push(name); // TODO extension?
            self.add_data(path, icon)?;
        }
        Ok(())
    }

    pub fn add_data<P: AsRef<Path>>(&mut self, path: P, data: Vec<u8>) -> Result<()> {
        self.set_header_and_append(path, data)?;
        Ok(())
    }

    pub fn finish(self) -> Result<()> {
        self.builder.into_inner()?.finish()?.flush()?;
        Ok(())
    }

    pub fn add_seed(&mut self, seed: &SeedWorld, pretty: bool) -> Result<()> {
        let ser = if pretty {
            serde_json::to_vec_pretty
        } else {
            serde_json::to_vec
        }; // TODO we might be able to make better assumptions about initial capacity
        let data = ser(&seed)?;
        self.set_header_and_append("seed", data)?;
        Ok(())
    }

    fn set_header_and_append<P: AsRef<Path>, D: AsRef<[u8]>>(
        &mut self,
        path: P,
        data: D,
    ) -> Result<()> {
        let data = data.as_ref();
        self.header.set_path(path)?;
        self.header.set_size(data.len().try_into()?);
        self.header.set_cksum();
        self.builder.append(&self.header, data)?;
        Ok(())
    }
}

fn base_header() -> tar::Header {
    let mut header = tar::Header::new_old();
    header.set_mode(0o644);
    header
}
