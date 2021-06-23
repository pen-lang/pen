mod build;
mod compile;
mod compile_configuration;
mod compile_dependency;
mod file_path_configuration;
mod main_package_directory_finder;

use build::build;
use compile::compile;
use compile_dependency::compile_dependency;

fn main() {
    if let Err(error) = run() {
        infra::log_error(error.as_ref()).unwrap();
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    match clap::App::new("pen")
        .version("0.1.0")
        .setting(clap::AppSettings::SubcommandRequired)
        .arg(
            clap::Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .global(true),
        )
        .subcommand(clap::SubCommand::with_name("build").about("Builds a package"))
        .subcommand(
            clap::SubCommand::with_name("compile")
                .about("Compiles a module")
                .arg(clap::Arg::with_name("source file").required(true))
                .arg(clap::Arg::with_name("dependency file").required(true))
                .arg(clap::Arg::with_name("object file").required(true))
                .arg(clap::Arg::with_name("interface file").required(true)),
        )
        .subcommand(
            clap::SubCommand::with_name("resolve-dependency")
                .about("Resolves module dependency")
                .arg(
                    clap::Arg::with_name("package directory")
                        .required(true)
                        .takes_value(true)
                        .short("p")
                        .long("package-directory"),
                )
                .arg(clap::Arg::with_name("source file").required(true))
                .arg(clap::Arg::with_name("object file").required(true))
                .arg(clap::Arg::with_name("dependency file").required(true))
                .arg(clap::Arg::with_name("build script dependency file").required(true)),
        )
        .get_matches()
        .subcommand()
    {
        ("build", matches) => build(matches.unwrap().is_present("verbose")),
        ("compile", matches) => {
            let matches = matches.unwrap();

            compile(
                matches.value_of("source file").unwrap(),
                matches.value_of("dependency file").unwrap(),
                matches.value_of("object file").unwrap(),
                matches.value_of("interface file").unwrap(),
            )
        }
        ("resolve-dependency", matches) => {
            let matches = matches.unwrap();

            compile_dependency(
                matches.value_of("package directory").unwrap(),
                matches.value_of("source file").unwrap(),
                matches.value_of("object file").unwrap(),
                matches.value_of("dependency file").unwrap(),
                matches.value_of("build script dependency file").unwrap(),
            )
        }
        _ => unreachable!(),
    }
}
