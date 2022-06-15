use clap::{Parser, Subcommand};

/// A fictional versioning CLI
#[derive(Debug, Parser)]
#[clap(name = "moeche")]
#[clap(author, version, about, long_about = None)]
#[clap(about = "A suite of tools to easily handle Rust->Python bindings", long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Builds a wheel given a building recipe
    #[clap(arg_required_else_help = true)]
    Wheel {
        /// The path to the recipe
        #[clap(value_parser)]
        recipe_path: String,
        /// Which target to build for of those defined in the recipe 
        #[clap(value_parser)]
        target: String,
        
        /// Where is the crate we want to compile, by default it's the current
        /// working directory.
        #[clap(long, value_parser)]
        crate_path: Option<String>,
    },
    /// Automatically generate the bindings from a rust crate
    #[clap(arg_required_else_help = true)]
    BindGen {
    },
    /// Automatically generate an harness to fuzz a rust crate
    #[clap(arg_required_else_help = true)]
    Harness {
    },
}



fn main() {
    let args = Cli::parse();
    match args.command {
        Commands::Wheel { recipe_path, target, crate_path } => {
            wheel_compiler::compile_wheel(
                recipe_path, 
                target, 
                crate_path.unwrap_or(std::env::current_dir().unwrap().display().to_string())
            ).unwrap();
        },
        Commands::Harness {..} => unimplemented!(),
        Commands::BindGen {..} => unimplemented!(),
    }
}
