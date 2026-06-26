use crate::{error::Error, model, plmn::Plmn, proto};
use anyhow::{Context, Result};
use std::{
    io::{Read, Write},
    path::Path,
};

fn read_input(path: Option<&Path>) -> Result<Vec<u8>> {
    match path {
        Some(p) => std::fs::read(p).with_context(|| format!("reading {}", p.display())),
        None => {
            let mut buf = Vec::new();
            std::io::stdin()
                .lock()
                .read_to_end(&mut buf)
                .context("reading stdin")?;
            Ok(buf)
        }
    }
}

fn write_output(path: Option<&Path>, data: &[u8]) -> Result<()> {
    match path {
        Some(p) => std::fs::write(p, data).with_context(|| format!("writing {}", p.display())),
        None => std::io::stdout()
            .lock()
            .write_all(data)
            .context("writing stdout"),
    }
}

pub fn decode(input: Option<&Path>, output: Option<&Path>) -> Result<()> {
    let bytes = read_input(input)?;
    let map = proto::parse(&bytes)?;
    let root = model::map_to_root(&map)?;
    let text = toml::to_string_pretty(&root).context("serializing TOML")?;
    write_output(output, text.as_bytes())
}

pub fn encode(input: Option<&Path>, output: Option<&Path>) -> Result<()> {
    let text = String::from_utf8(read_input(input)?).context("input is not UTF-8")?;
    let root: model::Root = toml::from_str(&text).context("parsing TOML")?;
    let map = model::root_to_map(&root)?;
    let bytes = proto::serialize(&map)?;
    write_output(output, &bytes)
}

pub fn inject(
    plmn: &str,
    mapping: &str,
    input: Option<&Path>,
    output: Option<&Path>,
) -> Result<()> {
    let value = plmn.parse::<Plmn>()?.to_encoded();
    let bytes = read_input(input)?;
    let mut map = proto::parse(&bytes)?;

    let entry = map
        .entry
        .iter_mut()
        .find(|e| e.identifier.as_deref() == Some(mapping))
        .ok_or_else(|| Error::MappingNotFound(mapping.to_string()))?;

    if !entry.plmns.contains(&value) {
        entry.plmns.push(value);
    }

    let out = proto::serialize(&map)?;
    write_output(output, &out)
}
