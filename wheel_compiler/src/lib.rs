use std::fs;
use std::io::prelude::*;
mod recipies_reader;
use recipies_reader::*;
use wheel_builder::{WheelBuilder, WheelSettings};

pub fn compile_wheel(recipe_path: String, target: String, crate_path: String) -> Result<(), String> {
    // get absoulte paths
    let crate_path = std::fs::canonicalize(&crate_path).unwrap().display().to_string();

    // try to read the recipie and get a standardize settings
    let recipe = Recipe::from_path(recipe_path)?;

    let recipe = recipe.validate()?;

    // TODO!: parse the crate_path/Cargo.toml to get the final versions
    let settings = WheelSettings {
        lib_name: "".into(),
        version: "".into(),
        python_tag: wheel_builder::PythonTag::CPython,
        abi_tag: "".into(),
        platform_tag: "".into(),
        python_minor_version: recipe.python_minor_version,
        dst_wheel_folder_path: recipe.dst_wheel_folder_path,
        readme_path: recipe.readme_path,
        requires_dist: recipe.requires_dist,
        keywords: recipe.keywords,
        authors: recipe.authors,
        author_emails: recipe.author_emails,
        license: recipe.license,
        project_url: recipe.project_url,
    };

    let mut wheel =  WheelBuilder::new(settings)?;
    let wheel_name = wheel.get_file_path().to_string();

    let builds = recipe.build_recipes.get(&target)
        .ok_or_else(||
            format!(
                "The given target '{}' is not one of the ones defined in the recipe: '{:?}'",
                target, recipe.build_recipes.keys().cloned().collect::<Vec<String>>(),
            )
        )?;

    for (target_name, target) in builds.targets.iter() {
        
        wheel.add_package_file(fs::File(compiled_target), dst_path)?;
    }

    // consolidate the wheel
    wheel.finish()?;

    if builds.install {
        //TODO!: pip install wheel_name
    }

    if builds.publish {
        //TODO!: twine upload wheel_name
    }

    Ok(())
}