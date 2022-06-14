mod application_configuration;
mod compile_configuration;
mod dependency_resolver;
mod documentation_configuration;
mod file_path_configuration;
mod infrastructure;
mod main_module_compiler;
mod main_package_directory_finder;
mod module_compiler;
mod module_formatter;
mod package_builder;
mod package_creator;
mod package_documentation_generator;
mod package_formatter;
mod package_test_information_compiler;
mod prelude_module_compiler;
mod test_configuration;
mod test_linker;
mod test_module_compiler;
mod test_runner;

use compile_configuration::CROSS_COMPILE_TARGETS;
use std::ops::Deref;

fn main() {
    if let Err(error) = run() {
        infra::log_error(error.as_ref()).unwrap();
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    match clap::Command::new("pen")
        .version(clap::crate_version!())
        .subcommand_required(true)
        .arg(
            clap::Arg::new("verbose")
                .short('v')
                .long("verbose")
                .global(true)
                .help("Use verbose output"),
        )
        .subcommand(clap::Command::new("build").about("Build a package").arg(
            build_target_triple_argument().value_parser(clap::builder::PossibleValuesParser::new(
                CROSS_COMPILE_TARGETS,
            )),
        ))
        .subcommand(clap::Command::new("test").about("Test modules in a package"))
        .subcommand(
            clap::Command::new("create")
                .about("Create a package")
                .arg(
                    clap::Arg::new("library")
                        .short('l')
                        .long("library")
                        .help("Create a library package instead of an application one"),
                )
                .arg(
                    clap::Arg::new("directory")
                        .required(true)
                        .help("Set a package directory"),
                ),
        )
        .subcommand(
            clap::Command::new("format")
                .about("Format a package")
                .arg(
                    clap::Arg::new("check")
                        .long("check")
                        .takes_value(false)
                        .help("Check if module files are formatted"),
                )
                .arg(
                    clap::Arg::new("stdin")
                        .long("stdin")
                        .takes_value(false)
                        .help("Format stdin"),
                ),
        )
        .subcommand(
            clap::Command::new("document")
                .about("Generate documentation for a package")
                .arg(
                    clap::Arg::new("name")
                        .long("name")
                        .takes_value(true)
                        .required(true)
                        .help("Set a package name"),
                )
                .arg(
                    clap::Arg::new("url")
                        .long("url")
                        .takes_value(true)
                        .required(true)
                        .help("Set a package URL"),
                )
                .arg(
                    clap::Arg::new("description")
                        .long("description")
                        .takes_value(true)
                        .required(true)
                        .help("Set package description"),
                ),
        )
        .subcommand(
            clap::Command::new("compile")
                .hide(true)
                .about("Compile a module")
                .arg(clap::Arg::new("source file").required(true))
                .arg(clap::Arg::new("dependency file").required(true))
                .arg(clap::Arg::new("object file").required(true))
                .arg(clap::Arg::new("interface file").required(true))
                .arg(build_target_triple_argument()),
        )
        .subcommand(
            clap::Command::new("compile-main")
                .hide(true)
                .about("Compile a main module")
                .arg(
                    clap::Arg::new("context interface file")
                        .short('c')
                        .long("context-interface-file")
                        .required(true)
                        .number_of_values(2)
                        .action(clap::ArgAction::Append),
                )
                .arg(clap::Arg::new("source file").required(true))
                .arg(clap::Arg::new("dependency file").required(true))
                .arg(clap::Arg::new("object file").required(true))
                .arg(build_target_triple_argument()),
        )
        .subcommand(
            clap::Command::new("compile-prelude")
                .hide(true)
                .about("Compile a prelude module")
                .arg(clap::Arg::new("source file").required(true))
                .arg(clap::Arg::new("object file").required(true))
                .arg(clap::Arg::new("interface file").required(true))
                .arg(build_target_triple_argument()),
        )
        .subcommand(
            clap::Command::new("compile-test")
                .hide(true)
                .about("Compile a test module")
                .arg(clap::Arg::new("source file").required(true))
                .arg(clap::Arg::new("dependency file").required(true))
                .arg(clap::Arg::new("object file").required(true))
                .arg(clap::Arg::new("test information file").required(true))
                .arg(build_target_triple_argument()),
        )
        .subcommand(
            clap::Command::new("resolve-dependency")
                .hide(true)
                .about("Resolve module dependency")
                .arg(
                    clap::Arg::new("package directory")
                        .short('p')
                        .long("package-directory")
                        .required(true)
                        .takes_value(true),
                )
                .arg(
                    clap::Arg::new("output directory")
                        .short('o')
                        .long("output-directory")
                        .required(true)
                        .takes_value(true),
                )
                .arg(
                    clap::Arg::new("prelude interface file")
                        .short('i')
                        .long("prelude-interface-file")
                        .action(clap::ArgAction::Append)
                        .number_of_values(1)
                        .takes_value(true),
                )
                .arg(clap::Arg::new("source file").required(true))
                .arg(clap::Arg::new("object file").required(true))
                .arg(clap::Arg::new("dependency file").required(true))
                .arg(clap::Arg::new("build script dependency file").required(true)),
        )
        .subcommand(
            clap::Command::new("compile-package-test-information")
                .hide(true)
                .about("Compile a package test information")
                .arg(
                    clap::Arg::new("package test information file")
                        .short('o')
                        .required(true)
                        .takes_value(true),
                )
                .arg(clap::Arg::new("test information file").multiple_values(true)),
        )
        .subcommand(
            clap::Command::new("link-test")
                .hide(true)
                .about("Link tests")
                .arg(
                    clap::Arg::new("test file")
                        .short('o')
                        .required(true)
                        .takes_value(true),
                )
                .arg(
                    clap::Arg::new("package test information file")
                        .short('i')
                        .required(true)
                        .takes_value(true),
                )
                .arg(
                    clap::Arg::new("archive file")
                        .required(true)
                        .multiple_values(true),
                ),
        )
        .get_matches()
        .subcommand()
        .unwrap()
    {
        ("build", matches) => package_builder::build(
            matches.get_one::<String>("target").map(Deref::deref),
            matches.contains_id("verbose"),
        ),
        ("test", _) => test_runner::run(),
        ("create", matches) => package_creator::create(
            matches.get_one::<String>("directory").unwrap(),
            matches.contains_id("library"),
        ),
        ("format", matches) => {
            if matches.contains_id("stdin") {
                module_formatter::format()
            } else {
                package_formatter::format(matches.contains_id("check"))
            }
        }
        ("document", matches) => package_documentation_generator::generate(
            matches.get_one::<String>("name").unwrap(),
            matches.get_one::<String>("url").unwrap(),
            matches.get_one::<String>("description").unwrap(),
        ),
        ("compile", matches) => module_compiler::compile(
            matches.get_one::<String>("source file").unwrap(),
            matches.get_one::<String>("dependency file").unwrap(),
            matches.get_one::<String>("object file").unwrap(),
            matches.get_one::<String>("interface file").unwrap(),
            matches.get_one::<String>("target").map(Deref::deref),
        ),
        ("compile-main", matches) => {
            let context_options = matches
                .get_many::<String>("context interface file")
                .unwrap()
                .map(Deref::deref)
                .collect::<Vec<_>>();

            main_module_compiler::compile(
                matches.get_one::<String>("source file").unwrap(),
                matches.get_one::<String>("dependency file").unwrap(),
                matches.get_one::<String>("object file").unwrap(),
                &context_options
                    .iter()
                    .step_by(2)
                    .map(Deref::deref)
                    .zip(context_options.iter().skip(1).step_by(2).map(Deref::deref))
                    .collect(),
                matches.get_one::<String>("target").map(Deref::deref),
            )
        }
        ("compile-prelude", matches) => prelude_module_compiler::compile(
            matches.get_one::<String>("source file").unwrap(),
            matches.get_one::<String>("object file").unwrap(),
            matches.get_one::<String>("interface file").unwrap(),
            matches.get_one::<String>("target").map(Deref::deref),
        ),
        ("compile-test", matches) => test_module_compiler::compile(
            matches.get_one::<String>("source file").unwrap(),
            matches.get_one::<String>("dependency file").unwrap(),
            matches.get_one::<String>("object file").unwrap(),
            matches.get_one::<String>("test information file").unwrap(),
            matches.get_one::<String>("target").map(Deref::deref),
        ),
        ("resolve-dependency", matches) => dependency_resolver::resolve(
            matches.get_one::<String>("source file").unwrap(),
            matches.get_one::<String>("object file").unwrap(),
            matches.get_one::<String>("dependency file").unwrap(),
            matches
                .get_one::<String>("build script dependency file")
                .unwrap(),
            &matches
                .get_many::<String>("prelude interface file")
                .unwrap()
                .map(Deref::deref)
                .collect::<Vec<_>>(),
            matches.get_one::<String>("package directory").unwrap(),
            matches.get_one::<String>("output directory").unwrap(),
        ),
        ("compile-package-test-information", matches) => {
            package_test_information_compiler::compile(
                &matches
                    .get_many::<String>("test information file")
                    .unwrap_or_default()
                    .map(Deref::deref)
                    .collect::<Vec<_>>(),
                matches
                    .get_one::<String>("package test information file")
                    .unwrap(),
            )
        }
        ("link-test", matches) => test_linker::link(
            &matches
                .get_many::<String>("archive file")
                .unwrap()
                .map(Deref::deref)
                .collect::<Vec<_>>(),
            matches
                .get_one::<String>("package test information file")
                .unwrap(),
            matches.get_one::<String>("test file").unwrap(),
        ),
        _ => unreachable!(),
    }
}

fn build_target_triple_argument() -> clap::Arg<'static> {
    clap::Arg::new("target")
        .short('t')
        .long("target")
        .takes_value(true)
        .help("Set a target triple")
}
