use super::*;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub(crate) struct BuildRecipe {
    pub(crate) install: Option<bool>,
    pub(crate) publish: Option<bool>,
    pub(crate) targets: Option<HashMap<String, BuildTarget>>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub(crate) struct ValidatedBuildRecipe {
    pub(crate) install: bool,
    pub(crate) publish: bool,
    pub(crate) targets: HashMap<String, ValidatedBuildTarget>,
}

impl Validate for BuildRecipe {
    type ValidatedType = ValidatedBuildRecipe;

    fn validate(self) -> Result<Self::ValidatedType, String> {
        Ok(ValidatedBuildRecipe{
            install: self.install.unwrap_or(false),
            publish: self.publish.unwrap_or(false),
            targets: self.targets.ok_or_else(|| "Missing targets in build-recipe".to_string())?
                .into_iter()
                .map(|(key, value)| Ok((key, value.validate()?)))
                .collect::<Result<HashMap<String, ValidatedBuildTarget>, String>>()?
        })
    }
}