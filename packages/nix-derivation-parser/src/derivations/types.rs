use std::path::PathBuf;

#[derive(Debug, PartialEq)]
pub struct DerivationOutput {
    pub key: String,
    pub path: PathBuf,
    pub hash_algo: String,
    pub hash: String,
}

#[derive(Debug, PartialEq)]
pub struct DerivationInput {
    pub key: PathBuf,
    pub value: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub struct Derivation {
    pub outputs: Vec<DerivationOutput>,
    pub input_drvs: Vec<DerivationInput>,
    pub input_srcs: Vec<PathBuf>,
    pub system: String,
    pub builder: PathBuf,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
}
