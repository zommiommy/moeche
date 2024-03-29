use std::{fs, hash::Hash};
use std::path::Path;
use std::collections::BTreeMap;
use serde::{Serialize, Deserialize};

mod build_recipe;
use build_recipe::*;
mod build_target;
use build_target::*;

pub(crate) trait Validate {
    type ValidatedType;
    fn validate(self) -> Result<Self::ValidatedType, String>;
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub(crate) struct ValidatedRecipe {
    pub(crate) python_minor_version: usize,
    pub(crate) dst_wheel_folder_path: String,
    pub(crate) build_folder: String,
    pub(crate) shared_rustflags: String,
    pub(crate) requires_dist: Vec<String>,
    pub(crate) build_recipes: BTreeMap<String, ValidatedBuildRecipe>,
}


#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub(crate) struct Recipe {
    pub(crate) include: Option<String>,
    pub(crate) python_minor_version: Option<usize>,
    pub(crate) dst_wheel_folder_path: Option<String>,
    pub(crate) build_folder: Option<String>,
    pub(crate) shared_rustflags: Option<String>,
    pub(crate) requires_dist: Option<Vec<String>>,
    pub(crate) target_triple: Option<String>,
    pub(crate) python_path: Option<String>,
    pub(crate) python_headers: Option<String>,
    pub(crate) build_recipes: Option<BTreeMap<String, BuildRecipe>>,
}


impl Validate for Recipe {
    type ValidatedType = ValidatedRecipe;

    fn validate(self) -> Result<Self::ValidatedType, String> {
        Ok(ValidatedRecipe{
            python_minor_version: self.python_minor_version.ok_or_else(|| "Missing python_minor_version in recipe".to_string())?,
            dst_wheel_folder_path: self.dst_wheel_folder_path.unwrap_or("/tmp/moeche/wheels".into()),
            build_folder: self.build_folder.unwrap_or("/tmp/moeche/build".into()),
            shared_rustflags: self.shared_rustflags.unwrap_or("".into()),
            requires_dist: self.requires_dist.unwrap_or_else(Vec::new),
            build_recipes: self.build_recipes.ok_or_else(|| "Missing build_recipes in recipe".to_string())?
                .into_iter()
                .map(|(key, value)| Ok((key, value.validate()?)))
                .collect::<Result<BTreeMap<String, ValidatedBuildRecipe>, String>>()?,
        })
    }
}

impl Recipe {
    pub(crate) fn from_path(path: String) -> Result<Self, String> {
        // use an absolute path so we don't get aweful errors
        let path = std::fs::canonicalize(&path).unwrap().display().to_string();
        // read the file
        let recipe_content = fs::read_to_string(&path)
        .map_err(|e| format!(
            "Could not read recipie at path: '{}'. The error is '{}'",
            path, e,
        ))?;
        // parse the yaml
        let mut recipie: Recipe = serde_yaml::from_str(&recipe_content)
        .map_err(|e| format!(
            "Can't read recipe at path: '{}'. The error is: '{}'",
            path, e,
        ))?;

        // resolve recusrion
        if let Some(include_path) = recipie.include.clone() {
            let path = Path::new(&path).parent().unwrap().join(include_path);
            let include = Recipe::from_path(path.display().to_string())?;
            recipie = recipie.set_defaults(include)?;
        }

        recipie.include = None;

        recipie.build_recipes = recipie.build_recipes.map(|brs| brs.into_iter()
            .map(|(key, mut value)| {
                value.targets = value.targets.map(|inner| {
                    inner.into_iter().map(|(k, mut v)| {
                        v.target_triple = v.target_triple.or(recipie.target_triple.clone());
                        v.python_path = v.python_path.or(recipie.python_path.clone());
                        v.python_headers = v.python_headers.or(recipie.python_headers.clone());
                        (k, v)
                    })
                    .collect::<BTreeMap<String, BuildTarget>>()
                });
                (
                    key, 
                    value
                )
            })
            .collect::<BTreeMap<String, BuildRecipe>>()
        );

        Ok(recipie)
    }

    /// Derive defaults from another recipe
    fn set_defaults(mut self, defaults: Recipe) -> Result<Self, String> {
        self.include = self.include.or(defaults.include);
        self.python_minor_version = self.python_minor_version.or(defaults.python_minor_version);
        self.dst_wheel_folder_path = self.dst_wheel_folder_path.or(defaults.dst_wheel_folder_path);
        self.build_folder = self.build_folder.or(defaults.build_folder);
        self.requires_dist = self.requires_dist.or(defaults.requires_dist);
        self.python_path = self.python_path.or(defaults.python_path);
        self.python_headers = self.python_headers.or(defaults.python_headers);
        self.build_recipes =  match (self.build_recipes, defaults.build_recipes) {
            (Some(orig), Some(mut defaults)) => {
                
                for (key, value) in orig.into_iter() {
                    if defaults.insert(key.clone(), value).is_some() {
                        return Err(format!(
                            "Duplicated build recipe '{}'",
                            key,
                        ));
                    }
                } 

                Some(defaults)
            },
            (Some(orig), None) => Some(orig),
            (None, Some(defaults)) => Some(defaults),
            (None, None) => None,
        };

        Ok(self)
    }
}
