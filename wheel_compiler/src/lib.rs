use std::fs;
use std::io::prelude::*;
use toml::Value;
use wheel_builder::{WheelBuilder, WheelSettings};

mod recipies_reader;
use recipies_reader::*;

pub fn compile_wheel(recipe_path: String, target: String, crate_path: String) -> Result<(), String> {
    // get absoulte paths
    let crate_path = std::fs::canonicalize(&crate_path).unwrap();
    let cargo_toml_path = crate_path.join("Cargo.toml");

    // parse the cargo toml to extract the metadata for the wheel
    let cargo_toml = fs::File::open(cargo_toml_path);
    if cargo_toml.is_err() {
        return Err(format!("Cargo.toml not foun in path {}", crate_path.display()));
    }
    let mut cargo_toml = cargo_toml.unwrap();
    let mut cargo_toml_str = String::new();
    cargo_toml.read_to_string(&mut cargo_toml_str).unwrap();

    let config = &cargo_toml_str.parse::<Value>().unwrap();
    let table = config.as_table().unwrap();
    let package = table.get("package").unwrap();
    let lib = table.get("lib").unwrap();

    println!("{:#4?}", package);
    println!("{:#4?}", lib);

    let mut readme_path = package.get("readme")
        .map(|x| x.as_str().unwrap().to_string());
    if crate_path.join("README.md").exists() {
        readme_path = Some(crate_path.join("README.md").display().to_string());
    }

    // try to read the recipie and get a standardize settings
    let recipe = Recipe::from_path(recipe_path)?;
    let recipe = recipe.validate()?;
    
    let settings = WheelSettings {
        lib_name: lib.get("name").unwrap().as_str().unwrap().into(),
        version: package.get("version").unwrap().as_str().unwrap().into(),

        python_tag: wheel_builder::PythonTag::CPython,
        abi_tag: "".into(),
        platform_tag: "".into(),
        python_minor_version: recipe.python_minor_version,
        dst_wheel_folder_path: recipe.dst_wheel_folder_path,
        requires_dist: recipe.requires_dist,

        readme_path: readme_path.unwrap(),

        license: lib.get("license").map(|x| x.as_str().unwrap().to_string()).unwrap_or(String::new()),
        project_url: lib.get("repository").map(|x| x.as_str().unwrap().to_string()),
        
        keywords: lib.get("keywords").map(|d| {
            d.as_array().unwrap().iter().map(|x| {
                x.as_str().unwrap().to_string()
            }).collect::<Vec<_>>()
        }).unwrap_or(vec![]),

        authors: vec![], // TODO parse the rust authrors into author and email
        author_emails: vec![], // TODO
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
        
        //wheel.add_package_file(fs::File(compiled_target), dst_path)?;
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