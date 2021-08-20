use compile_configuration::CROSS_COMPILE_TARGETS;

mod application_configuration;
mod compile_configuration;
mod dependency_resolver;
mod file_path_configuration;
mod infrastructure;
mod main_module_compiler;
mod main_package_directory_finder;
mod module_compiler;
mod package_builder;
mod package_creator;
mod prelude_module_compiler;

fn main() {
    if let Err(error) = run() {
        infra::log_error(error.as_ref()).unwrap();
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    match clap::App::new("pen")
        .version(clap::crate_version!())
        .setting(clap::AppSettings::SubcommandRequired)
        .arg(
            clap::Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .global(true)
                .help("Uses verbose output"),
        )
        .subcommand(
            clap::SubCommand::with_name("build")
                .about("Builds a package")
                .arg(build_target_triple_argument()),
        )
        .subcommand(
            clap::SubCommand::with_name("create")
                .about("Creates a package")
                .arg(
                    clap::Arg::with_name("library")
                        .short("l")
                        .long("library")
                        .help("Creates a library package instead of an application one"),
                )
                .arg(
                    clap::Arg::with_name("directory")
                        .required(true)
                        .help("Sets a package directory"),
                ),
        )
        .subcommand(
            clap::SubCommand::with_name("compile")
                .setting(clap::AppSettings::Hidden)
                .about("Compiles a module")
                .arg(clap::Arg::with_name("source file").required(true))
                .arg(clap::Arg::with_name("dependency file").required(true))
                .arg(clap::Arg::with_name("object file").required(true))
                .arg(clap::Arg::with_name("interface file").required(true))
                .arg(build_target_triple_argument()),
        )
        .subcommand(
            clap::SubCommand::with_name("compile-main")
                .setting(clap::AppSettings::Hidden)
                .about("Compiles a main module")
                .arg(
                    clap::Arg::with_name("main function interface file")
                        .short("f")
                        .long("main-function-interface-file")
                        .required(true)
                        .takes_value(true),
                )
                .arg(clap::Arg::with_name("source file").required(true))
                .arg(clap::Arg::with_name("dependency file").required(true))
                .arg(clap::Arg::with_name("object file").required(true))
                .arg(build_target_triple_argument()),
        )
        .subcommand(
            clap::SubCommand::with_name("compile-prelude")
                .setting(clap::AppSettings::Hidden)
                .about("Compiles a prelude module")
                .arg(clap::Arg::with_name("source file").required(true))
                .arg(clap::Arg::with_name("object file").required(true))
                .arg(clap::Arg::with_name("interface file").required(true))
                .arg(build_target_triple_argument()),
        )
        .subcommand(
            clap::SubCommand::with_name("resolve-dependency")
                .setting(clap::AppSettings::Hidden)
                .about("Resolves module dependency")
                .arg(
                    clap::Arg::with_name("package directory")
                        .short("p")
                        .long("package-directory")
                        .required(true)
                        .takes_value(true),
                )
                .arg(
                    clap::Arg::with_name("output directory")
                        .short("o")
                        .long("output-directory")
                        .required(true)
                        .takes_value(true),
                )
                .arg(
                    clap::Arg::with_name("prelude interface file")
                        .short("i")
                        .long("prelude-interface-file")
                        .multiple(true)
                        .number_of_values(1)
                        .takes_value(true),
                )
                .arg(clap::Arg::with_name("source file").required(true))
                .arg(clap::Arg::with_name("object file").required(true))
                .arg(clap::Arg::with_name("dependency file").required(true))
                .arg(clap::Arg::with_name("build script dependency file").required(true)),
        )
        .get_matches()
        .subcommand()
    {
        ("build", matches) => {
            let matches = matches.unwrap();

            package_builder::build(matches.value_of("target"), matches.is_present("verbose"))
        }
        ("create", matches) => {
            let matches = matches.unwrap();

            package_creator::create(
                matches.value_of("directory").unwrap(),
                matches.is_present("library"),
            )
        }
        ("compile", matches) => {
            let matches = matches.unwrap();

            module_compiler::compile(
                matches.value_of("source file").unwrap(),
                matches.value_of("dependency file").unwrap(),
                matches.value_of("object file").unwrap(),
                matches.value_of("interface file").unwrap(),
                matches.value_of("target"),
            )
        }
        ("compile-main", matches) => {
            let matches = matches.unwrap();

            main_module_compiler::compile(
                matches.value_of("source file").unwrap(),
                matches.value_of("dependency file").unwrap(),
                matches.value_of("object file").unwrap(),
                matches.value_of("main function interface file").unwrap(),
                matches.value_of("target"),
            )
        }
        ("compile-prelude", matches) => {
            let matches = matches.unwrap();

            prelude_module_compiler::compile(
                matches.value_of("source file").unwrap(),
                matches.value_of("object file").unwrap(),
                matches.value_of("interface file").unwrap(),
                matches.value_of("target"),
            )
        }
        ("resolve-dependency", matches) => {
            let matches = matches.unwrap();

            dependency_resolver::resolve(
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
            )
        }
        _ => unreachable!(),
    }
}

fn build_target_triple_argument() -> clap::Arg<'static, 'static> {
    clap::Arg::with_name("target")
        .short("t")
        .long("target")
        .takes_value(true)
        .possible_values(CROSS_COMPILE_TARGETS)
        .help("Sets a target triple")
}
