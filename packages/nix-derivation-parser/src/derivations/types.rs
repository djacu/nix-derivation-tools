use std::collections::HashMap;
use std::path::PathBuf;

#[expect(clippy::exhaustive_structs, reason = "Derivation format is very stable.")]
#[derive(Debug, Eq, Hash, PartialEq)]
pub struct DerivationOutput {
    pub path: PathBuf,
    pub hash_algo: String,
    pub hash: String,
}

#[expect(clippy::exhaustive_structs, reason = "Derivation format is very stable.")]
#[derive(Debug, PartialEq)]
pub struct DerivationInput {
    pub value: Vec<String>,
}

#[expect(clippy::exhaustive_structs, reason = "Derivation format is very stable.")]
#[derive(Debug, PartialEq)]
pub struct Derivation {
    pub outputs: HashMap<String, DerivationOutput>,
    pub input_drvs: HashMap<PathBuf, DerivationInput>,
    pub input_srcs: Vec<PathBuf>,
    pub system: String,
    pub builder: PathBuf,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
}
