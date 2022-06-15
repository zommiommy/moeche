use super::*;
const TARGET_TRIPLE: &str = env!("TARGET_TRIPLE");

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub(crate) struct BuildTarget {
    pub(crate) target_cpu: Option<String>,
    pub(crate) features: Option<Vec<String>>,
    pub(crate) target_triple: Option<String>,
    pub(crate) python_path: Option<String>,
    pub(crate) python_headers: Option<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub(crate) struct ValidatedBuildTarget {
    pub(crate) target_cpu: String,
    pub(crate) features: Vec<String>,
    pub(crate) target_triple: String,
    pub(crate) python_path: Option<String>,
    pub(crate) python_headers: Option<String>,
}

impl Validate for BuildTarget {
    type ValidatedType = ValidatedBuildTarget;

    fn validate(self) -> Result<Self::ValidatedType, String> {
        Ok(ValidatedBuildTarget{
            target_cpu: self.target_cpu.ok_or_else(|| "Missing target_cpu in recipe".to_string())?,
            target_triple: self.target_triple.unwrap_or(TARGET_TRIPLE.to_string()),

            features: self.features.unwrap_or_else(Vec::new),
            python_path: self.python_path,
            python_headers: self.python_headers,
        })
    }
}