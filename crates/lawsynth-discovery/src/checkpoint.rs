use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

use lawsynth_core::Identifier;

use crate::DiscoveryError;

const MAGIC: &str = "LSCP2";
const LEGACY_MAGIC: &str = "LSCP1";

/// A fitted sparse law retained for deterministic checkpoint resumption.
#[derive(Clone, Debug, PartialEq)]
pub struct CheckpointLaw {
    pub expression: String,
    pub residual_sum_squares: f64,
}

/// Durable progress metadata for a discovery execution.
#[derive(Clone, Debug, PartialEq)]
pub struct DiscoveryCheckpoint {
    dataset_fingerprint: u64,
    configuration_fingerprint: Option<u64>,
    completed_states: BTreeSet<Identifier>,
    laws: BTreeMap<Identifier, CheckpointLaw>,
}

impl DiscoveryCheckpoint {
    pub fn new(dataset_fingerprint: u64) -> Self {
        Self {
            dataset_fingerprint,
            configuration_fingerprint: None,
            completed_states: BTreeSet::new(),
            laws: BTreeMap::new(),
        }
    }

    pub fn dataset_fingerprint(&self) -> u64 {
        self.dataset_fingerprint
    }

    pub fn completed_states(&self) -> impl Iterator<Item = &Identifier> {
        self.completed_states.iter()
    }

    pub fn law(&self, state: &Identifier) -> Option<&CheckpointLaw> {
        self.laws.get(state)
    }

    pub fn record_state(&mut self, state: Identifier) {
        self.completed_states.insert(state);
    }

    pub fn record_law(&mut self, state: Identifier, expression: String, residual_sum_squares: f64) {
        self.completed_states.insert(state.clone());
        self.laws.insert(state, CheckpointLaw { expression, residual_sum_squares });
    }

    pub fn is_compatible_with(&self, dataset_fingerprint: u64) -> bool {
        self.dataset_fingerprint == dataset_fingerprint
    }

    pub fn ensure_configuration(&mut self, configuration_fingerprint: u64) -> bool {
        match self.configuration_fingerprint {
            Some(existing) => existing == configuration_fingerprint,
            None => {
                self.configuration_fingerprint = Some(configuration_fingerprint);
                true
            }
        }
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), DiscoveryError> {
        let mut contents = format!(
            "{MAGIC}\n{}\n{}\n",
            self.dataset_fingerprint,
            self.configuration_fingerprint
                .map(|value| value.to_string())
                .unwrap_or_else(|| "-".to_owned())
        );
        for state in &self.completed_states {
            if let Some(law) = self.laws.get(state) {
                contents.push_str("L\t");
                contents.push_str(state.as_str());
                contents.push('\t');
                contents.push_str(&law.residual_sum_squares.to_bits().to_string());
                contents.push('\t');
                contents.push_str(&law.expression);
                contents.push('\n');
            } else {
                contents.push_str("S\t");
                contents.push_str(state.as_str());
                contents.push('\n');
            }
        }
        fs::write(path, contents).map_err(|error| DiscoveryError::Checkpoint(error.to_string()))
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self, DiscoveryError> {
        let contents = fs::read_to_string(path)
            .map_err(|error| DiscoveryError::Checkpoint(error.to_string()))?;
        let mut lines = contents.lines();
        let magic = lines.next();
        if magic == Some(LEGACY_MAGIC) {
            return load_legacy(lines);
        }
        if magic != Some(MAGIC) {
            return Err(DiscoveryError::Checkpoint("unsupported checkpoint version".to_owned()));
        }
        let dataset_fingerprint = lines
            .next()
            .ok_or_else(|| DiscoveryError::Checkpoint("missing dataset fingerprint".to_owned()))?
            .parse()
            .map_err(|_| DiscoveryError::Checkpoint("invalid dataset fingerprint".to_owned()))?;
        let configuration_fingerprint = match lines.next() {
            Some("-") => None,
            Some(value) => Some(value.parse().map_err(|_| {
                DiscoveryError::Checkpoint("invalid configuration fingerprint".to_owned())
            })?),
            None => {
                return Err(DiscoveryError::Checkpoint(
                    "missing configuration fingerprint".to_owned(),
                ));
            }
        };
        let mut completed_states = BTreeSet::new();
        let mut laws = BTreeMap::new();
        for line in lines {
            let mut fields = line.splitn(4, '\t');
            match fields.next() {
                Some("S") => {
                    let state = parse_state(fields.next())?;
                    if fields.next().is_some() {
                        return Err(DiscoveryError::Checkpoint("invalid state record".to_owned()));
                    }
                    completed_states.insert(state);
                }
                Some("L") => {
                    let state = parse_state(fields.next())?;
                    let residual_sum_squares = f64::from_bits(
                        fields
                            .next()
                            .ok_or_else(|| {
                                DiscoveryError::Checkpoint("missing residual".to_owned())
                            })?
                            .parse()
                            .map_err(|_| {
                                DiscoveryError::Checkpoint("invalid residual".to_owned())
                            })?,
                    );
                    let expression = fields
                        .next()
                        .ok_or_else(|| DiscoveryError::Checkpoint("missing expression".to_owned()))?
                        .to_owned();
                    completed_states.insert(state.clone());
                    laws.insert(state, CheckpointLaw { expression, residual_sum_squares });
                }
                _ => {
                    return Err(DiscoveryError::Checkpoint("invalid checkpoint record".to_owned()));
                }
            }
        }
        Ok(Self { dataset_fingerprint, configuration_fingerprint, completed_states, laws })
    }
}

fn load_legacy<'a>(
    lines: impl Iterator<Item = &'a str>,
) -> Result<DiscoveryCheckpoint, DiscoveryError> {
    let mut lines = lines;
    let dataset_fingerprint = lines
        .next()
        .ok_or_else(|| DiscoveryError::Checkpoint("missing dataset fingerprint".to_owned()))?
        .parse()
        .map_err(|_| DiscoveryError::Checkpoint("invalid dataset fingerprint".to_owned()))?;
    let completed_states = lines
        .map(|state| {
            Identifier::new(state).map_err(|error| DiscoveryError::Checkpoint(error.to_string()))
        })
        .collect::<Result<BTreeSet<_>, _>>()?;
    Ok(DiscoveryCheckpoint {
        dataset_fingerprint,
        configuration_fingerprint: None,
        completed_states,
        laws: BTreeMap::new(),
    })
}

fn parse_state(field: Option<&str>) -> Result<Identifier, DiscoveryError> {
    let value = field.ok_or_else(|| DiscoveryError::Checkpoint("missing state".to_owned()))?;
    Identifier::new(value).map_err(|error| DiscoveryError::Checkpoint(error.to_string()))
}

#[cfg(test)]
mod tests {
    use lawsynth_core::Identifier;

    use super::*;

    #[test]
    fn checkpoint_round_trips_in_stable_order() {
        let mut checkpoint = DiscoveryCheckpoint::new(42);
        checkpoint.record_state(Identifier::new("z").unwrap());
        checkpoint.record_law(Identifier::new("x").unwrap(), "(2.0e0*x)".to_owned(), 0.125);
        let path = std::env::temp_dir().join(format!("lawsynth-checkpoint-{}", std::process::id()));
        checkpoint.save(&path).unwrap();
        assert_eq!(DiscoveryCheckpoint::load(&path).unwrap(), checkpoint);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn legacy_checkpoint_stays_loadable() {
        let path =
            std::env::temp_dir().join(format!("lawsynth-legacy-checkpoint-{}", std::process::id()));
        fs::write(&path, "LSCP1\n42\nx\n").unwrap();
        let checkpoint = DiscoveryCheckpoint::load(&path).unwrap();
        assert!(checkpoint.law(&Identifier::new("x").unwrap()).is_none());
        fs::remove_file(path).unwrap();
    }
}
