use crate::{
    error::Error,
    plmn::Plmn,
    proto::{CarrierEntry, CarrierMap},
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Serialize, Deserialize)]
pub struct Root {
    #[serde(default, rename = "mapping")]
    pub mappings: Vec<MappingEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MappingEntry {
    pub id: i32,
    pub name: String,
    pub plmns: Vec<String>,
}

/// proto → editable model (decode). Preserves entry and `plmns` order verbatim.
pub fn map_to_root(map: &CarrierMap) -> Result<Root, Error> {
    let mut mappings = Vec::with_capacity(map.entry.len());
    for (index, e) in map.entry.iter().enumerate() {
        let id = e.carrier_id.ok_or(Error::MissingField {
            index,
            field: "carrier_id",
        })?;
        let name = e.identifier.clone().ok_or(Error::MissingField {
            index,
            field: "identifier",
        })?;
        let mut plmns = Vec::with_capacity(e.plmns.len());
        for &v in &e.plmns {
            plmns.push(Plmn::from_encoded(v)?.to_string());
        }
        mappings.push(MappingEntry { id, name, plmns });
    }
    Ok(Root { mappings })
}

/// editable model → proto (encode). Validates unique ids/names; keeps order and
/// duplicate PLMNs verbatim.
pub fn root_to_map(root: &Root) -> Result<CarrierMap, Error> {
    let mut seen_ids: HashSet<i32> = HashSet::new();
    let mut seen_names: HashSet<&str> = HashSet::new();
    Ok(CarrierMap {
        entry: root
            .mappings
            .iter()
            .map(|MappingEntry { id, name, plmns }| {
                if !seen_ids.insert(*id) {
                    return Err(Error::DuplicateId(*id));
                }
                if !seen_names.insert(name.as_str()) {
                    return Err(Error::DuplicateName(name.clone()));
                }
                let plmns = plmns
                    .iter()
                    .map(|s| Ok(s.parse::<Plmn>()?.to_encoded()))
                    .collect::<Result<Vec<_>, Error>>()?;

                Ok(CarrierEntry {
                    plmns,
                    carrier_id: Some(*id),
                    identifier: Some(name.clone()),
                    ..Default::default()
                })
            })
            .collect::<Result<_, Error>>()?,
        ..Default::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_duplicate_id() {
        let root = Root {
            mappings: vec![
                MappingEntry {
                    id: 1,
                    name: "A".into(),
                    plmns: vec![],
                },
                MappingEntry {
                    id: 1,
                    name: "B".into(),
                    plmns: vec![],
                },
            ],
        };
        assert!(matches!(root_to_map(&root), Err(Error::DuplicateId(1))));
    }

    #[test]
    fn rejects_duplicate_name() {
        let root = Root {
            mappings: vec![
                MappingEntry {
                    id: 1,
                    name: "A".into(),
                    plmns: vec![],
                },
                MappingEntry {
                    id: 2,
                    name: "A".into(),
                    plmns: vec![],
                },
            ],
        };
        assert!(matches!(root_to_map(&root), Err(Error::DuplicateName(_))));
    }
}
