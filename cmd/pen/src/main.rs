mod application_configuration;
mod compile_configuration;
mod dependency_resolver;
mod file_path_configuration;
mod infrastructure;
mod main_module_compiler;
mod main_package_directory_finder;
mod module_compiler;
mod module_formatter;
mod package_builder;
mod package_creator;
mod package_formatter;
mod package_test_information_compiler;
mod prelude_module_compiler;
mod test_configuration;
mod test_linker;
mod test_module_compiler;
mod test_runner;

use compile_configuration::CROSS_COMPILE_TARGETS;

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
                .help("Uses verbose output"),
        )
        .subcommand(
            clap::Command::new("build")
                .about("Builds a package")
                .arg(build_target_triple_argument().possible_values(CROSS_COMPILE_TARGETS)),
        )
        .subcommand(clap::Command::new("test").about("Tests modules in a package"))
        .subcommand(
            clap::Command::new("create")
                .about("Creates a package")
                .arg(
                    clap::Arg::new("library")
                        .short('l')
                        .long("library")
                        .help("Creates a library package instead of an application one"),
                )
                .arg(
                    clap::Arg::new("directory")
                        .required(true)
                        .help("Sets a package directory"),
                ),
        )
        .subcommand(
            clap::Command::new("format")
                .about("Formats a package")
                .arg(
                    clap::Arg::new("check")
                        .long("check")
                        .takes_value(false)
                        .help("Checks if module files are formatted"),
                )
                .arg(
                    clap::Arg::new("stdin")
                        .long("stdin")
                        .takes_value(false)
                        .help("Formats stdin"),
                ),
        )
        .subcommand(
            clap::Command::new("compile")
                .hide(true)
                .about("Compiles a module")
                .arg(clap::Arg::new("source file").required(true))
                .arg(clap::Arg::new("dependency file").required(true))
                .arg(clap::Arg::new("object file").required(true))
                .arg(clap::Arg::new("interface file").required(true))
                .arg(build_target_triple_argument()),
        )
        .subcommand(
            clap::Command::new("compile-main")
                .hide(true)
                .about("Compiles a main module")
                .arg(
                    clap::Arg::new("context interface file")
                        .short('c')
                        .long("context-interface-file")
                        .required(true)
                        .number_of_values(2)
                        .multiple_occurrences(true),
                )
                .arg(clap::Arg::new("source file").required(true))
                .arg(clap::Arg::new("dependency file").required(true))
                .arg(clap::Arg::new("object file").required(true))
                .arg(build_target_triple_argument()),
        )
        .subcommand(
            clap::Command::new("compile-prelude")
                .hide(true)
                .about("Compiles a prelude module")
                .arg(clap::Arg::new("source file").required(true))
                .arg(clap::Arg::new("object file").required(true))
                .arg(clap::Arg::new("interface file").required(true))
                .arg(build_target_triple_argument()),
        )
        .subcommand(
            clap::Command::new("compile-test")
                .hide(true)
                .about("Compiles a test module")
                .arg(clap::Arg::new("source file").required(true))
                .arg(clap::Arg::new("dependency file").required(true))
                .arg(clap::Arg::new("object file").required(true))
                .arg(clap::Arg::new("test information file").required(true))
                .arg(build_target_triple_argument()),
        )
        .subcommand(
            clap::Command::new("resolve-dependency")
                .hide(true)
                .about("Resolves module dependency")
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
                        .multiple_occurrences(true)
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
                .about("Compiles a package test information")
                .arg(
                    clap::Arg::new("package test information file")
                        .short('o')
                        .required(true)
                        .takes_value(true),
                )
                .arg(clap::Arg::new("test information file").multiple_occurrences(true)),
        )
        .subcommand(
            clap::Command::new("link-test")
                .hide(true)
                .about("Links tests")
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
                        .multiple_occurrences(true)
                        .required(true)
                        .takes_value(true),
                ),
        )
        .get_matches()
        .subcommand()
        .unwrap()
    {
        ("build", matches) => {
            package_builder::build(matches.value_of("target"), matches.is_present("verbose"))
        }
        ("test", _) => test_runner::run(),
        ("create", matches) => package_creator::create(
            matches.value_of("directory").unwrap(),
            matches.is_present("library"),
        ),
        ("format", matches) => {
            if matches.is_present("stdin") {
                module_formatter::format()
            } else {
                package_formatter::format(matches.is_present("check"))
            }
        }
        ("compile", matches) => module_compiler::compile(
            matches.value_of("source file").unwrap(),
            matches.value_of("dependency file").unwrap(),
            matches.value_of("object file").unwrap(),
            matches.value_of("interface file").unwrap(),
            matches.value_of("target"),
        ),
        ("compile-main", matches) => {
            let context_options = matches
                .values_of("context interface file")
                .unwrap()
                .collect::<Vec<_>>();

            main_module_compiler::compile(
                matches.value_of("source file").unwrap(),
                matches.value_of("dependency file").unwrap(),
                matches.value_of("object file").unwrap(),
                &context_options
                    .iter()
                    .step_by(2)
                    .cloned()
                    .zip(context_options.iter().skip(1).step_by(2).cloned())
                    .collect(),
                matches.value_of("target"),
            )
        }
        ("compile-prelude", matches) => prelude_module_compiler::compile(
            matches.value_of("source file").unwrap(),
            matches.value_of("object file").unwrap(),
            matches.value_of("interface file").unwrap(),
            matches.value_of("target"),
        ),
        ("compile-test", matches) => test_module_compiler::compile(
            matches.value_of("source file").unwrap(),
            matches.value_of("dependency file").unwrap(),
            matches.value_of("object file").unwrap(),
            matches.value_of("test information file").unwrap(),
            matches.value_of("target"),
        ),
        ("resolve-dependency", matches) => dependency_resolver::resolve(
            matches.value_of("source file").unwrap(),
            matches.value_of("object file").unwrap(),
            matches.value_of("dependency file").unwrap(),
            matches.value_of("build script dependency file").unwrap(),
            &matches
                .values_of("prelude interface file")
                .unwrap()
                .collect::<Vec<_>>(),
            matches.value_of("package directory").unwrap(),
            matches.value_of("output directory").unwrap(),
        ),
        ("compile-package-test-information", matches) => {
            package_test_information_compiler::compile(
                &matches
                    .values_of("test information file")
                    .unwrap_or_default()
                    .collect::<Vec<_>>(),
                matches.value_of("package test information file").unwrap(),
            )
        }
        ("link-test", matches) => test_linker::link(
            &matches
                .values_of("archive file")
                .unwrap()
                .collect::<Vec<_>>(),
            matches.value_of("package test information file").unwrap(),
            matches.value_of("test file").unwrap(),
        ),
        _ => unreachable!(),
    }
}

fn build_target_triple_argument() -> clap::Arg<'static> {
    clap::Arg::new("target")
        .short('t')
        .long("target")
        .takes_value(true)
        .help("Sets a target triple")
}
