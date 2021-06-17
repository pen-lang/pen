mod compile;
mod compile_configuration;

use compile::compile;

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
        .subcommand(
            clap::SubCommand::with_name("compile")
                .about("Compiles a module")
                .arg(
                    clap::Arg::with_name("package prefix")
                        .required(true)
                        .short("p")
                        .long("package-prefix")
                        .help("Sets a package prefix"),
                )
                .arg(
                    clap::Arg::with_name("module prefix")
                        .required(true)
                        .short("m")
                        .long("module-prefix")
                        .help("Sets a module prefix"),
                )
                .arg(
                    clap::Arg::with_name("source path")
                        .required(true)
                        .help("source path"),
                )
                .arg(
                    clap::Arg::with_name("object path")
                        .required(true)
                        .help("object path"),
                ),
        )
        .get_matches()
        .subcommand()
    {
        ("compile", matches) => {
            let matches = matches.unwrap();

            compile(
                matches.value_of("source path").unwrap(),
                matches.value_of("object path").unwrap(),
                matches.value_of("module prefix").unwrap(),
                matches.value_of("package prefix").unwrap(),
            )
        }
        _ => unreachable!(),
    }
}
