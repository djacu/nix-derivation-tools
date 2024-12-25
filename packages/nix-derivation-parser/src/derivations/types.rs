use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Eq, Hash, PartialEq)]
pub struct DerivationOutput {
    pub path: PathBuf,
    pub hash_algo: String,
    pub hash: String,
}

#[derive(Debug, PartialEq)]
pub struct DerivationInput {
    pub value: Vec<String>,
}

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
