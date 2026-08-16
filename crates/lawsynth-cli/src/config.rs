/// Process-neutral command configuration shared by embedders and the binary.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CliConfig {
    pub color: bool,
    pub quiet: bool,
}
