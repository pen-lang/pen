use super::file_path_converter::FilePathConverter;
use rlimit::{Resource, increase_nofile_limit};
use std::{error::Error, path::Path, rc::Rc, sync::Arc, thread::available_parallelism};
use tokio::{runtime::Runtime, sync::Mutex};
use turtle_build::{
    BuildError, Console, Context, LogDatabase, OsCommandRunner, OsConsole, OsFileSystem, PathPool,
    RunOptions, clean_dead, compile, job_limit, load, run,
};

const DATABASE_DIRECTORY: &str = ".turtle";
const DEFAULT_FILE_COUNT_PER_PROCESS: usize = 3; // stdin, stdout, and stderr

pub struct NinjaBuildScriptRunner {
    file_path_converter: Rc<FilePathConverter>,
    database_version: &'static str,
}

impl NinjaBuildScriptRunner {
    pub fn new(file_path_converter: Rc<FilePathConverter>, database_version: &'static str) -> Self {
        Self {
            file_path_converter,
            database_version,
        }
    }

    async fn build(
        &self,
        build_script_file: &Path,
        target_file: &Path,
        console: &Arc<Mutex<OsConsole>>,
    ) -> Result<(), BuildError> {
        increase_nofile_limit(u64::MAX)?;

        let job_limit = job_limit(available_parallelism().map(usize::from).unwrap_or_default());
        let file_system = OsFileSystem::new(
            usize::try_from(Resource::NOFILE.get_soft()?)
                .unwrap_or(usize::MAX)
                .saturating_sub(DEFAULT_FILE_COUNT_PER_PROCESS * (job_limit + 1))
                .max(1),
        );
        let modules = load(&file_system, build_script_file).await?;
        let path_pool = Arc::new(PathPool::new());
        let config = Arc::new(compile(&modules, build_script_file, &path_pool)?);
        let context = Arc::new(Context::new(
            OsCommandRunner::new(job_limit),
            console.clone(),
            LogDatabase::new(
                &config
                    .build_directory()
                    .map(|directory| Path::new(directory.as_ref()).join(DATABASE_DIRECTORY))
                    .unwrap_or_else(|| build_script_file.with_file_name(DATABASE_DIRECTORY))
                    .join(self.database_version),
                &path_pool,
            )
            .await?,
            file_system,
            path_pool,
        ));

        clean_dead(&context, &config).await?;

        run(
            &context,
            config,
            &[target_file.to_string_lossy().into()],
            RunOptions {
                debug: false,
                dry_run: false,
                profile: false,
            },
        )
        .await
    }
}

impl app::infra::BuildScriptRunner for NinjaBuildScriptRunner {
    fn run(
        &self,
        build_script_file: &app::infra::FilePath,
        target_file: &app::infra::FilePath,
    ) -> Result<(), Box<dyn Error>> {
        Runtime::new()?.block_on(async {
            let console = Arc::new(Mutex::new(OsConsole::new()));
            let result = self
                .build(
                    &self
                        .file_path_converter
                        .convert_to_os_path(build_script_file),
                    &self.file_path_converter.convert_to_os_path(target_file),
                    &console,
                )
                .await;

            // Build jobs report their own failures.
            if let Err(error) = &result
                && !matches!(error, BuildError::Build)
            {
                console
                    .lock()
                    .await
                    .write_stderr(format!("{error}\n").as_bytes())
                    .await?;
            }

            console.lock().await.flush().await?;

            result
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use app::infra::{BuildScriptRunner, FilePath};
    use std::fs::{read_to_string, write};
    use tempfile::tempdir;

    const BUILD_SCRIPT_FILENAME: &str = "build.ninja";

    fn run_build_script(
        directory: &Path,
        build_script: &str,
        target: &str,
    ) -> Result<(), Box<dyn Error>> {
        write(directory.join(BUILD_SCRIPT_FILENAME), build_script)?;

        NinjaBuildScriptRunner::new(FilePathConverter::new(directory).into(), "1.2.3").run(
            &FilePath::new([BUILD_SCRIPT_FILENAME]),
            &FilePath::new([target]),
        )
    }

    fn compile_build_script(directory: &Path, outputs: &[&str]) -> String {
        format!(
            "builddir = {}\nrule echo\n  command = echo $out > $out\n",
            directory.join("build").display()
        ) + &outputs
            .iter()
            .map(|output| format!("build {}: echo\n", directory.join(output).display()))
            .collect::<String>()
    }

    #[test]
    fn build_target() {
        let directory = tempdir().unwrap();

        run_build_script(
            directory.path(),
            &compile_build_script(directory.path(), &["foo"]),
            "foo",
        )
        .unwrap();

        assert_eq!(
            read_to_string(directory.path().join("foo")).unwrap(),
            format!("{}\n", directory.path().join("foo").display())
        );
    }

    #[test]
    fn build_only_target() {
        let directory = tempdir().unwrap();

        run_build_script(
            directory.path(),
            &compile_build_script(directory.path(), &["foo", "bar"]),
            "foo",
        )
        .unwrap();

        assert!(directory.path().join("foo").exists());
        assert!(!directory.path().join("bar").exists());
    }

    #[test]
    fn store_database_in_build_directory() {
        let directory = tempdir().unwrap();

        run_build_script(
            directory.path(),
            &compile_build_script(directory.path(), &["foo"]),
            "foo",
        )
        .unwrap();

        assert!(
            directory
                .path()
                .join("build")
                .join(DATABASE_DIRECTORY)
                .join("1.2.3")
                .is_dir()
        );
    }

    #[test]
    fn store_database_next_to_build_script() {
        let directory = tempdir().unwrap();

        run_build_script(
            directory.path(),
            &format!(
                "rule touch\n  command = touch $out\nbuild {}: touch\n",
                directory.path().join("foo").display()
            ),
            "foo",
        )
        .unwrap();

        assert!(
            directory
                .path()
                .join(DATABASE_DIRECTORY)
                .join("1.2.3")
                .is_dir()
        );
    }

    #[test]
    fn clean_dead_output() {
        let directory = tempdir().unwrap();

        run_build_script(
            directory.path(),
            &compile_build_script(directory.path(), &["foo"]),
            "foo",
        )
        .unwrap();
        run_build_script(
            directory.path(),
            &compile_build_script(directory.path(), &["bar"]),
            "bar",
        )
        .unwrap();

        assert!(!directory.path().join("foo").exists());
        assert!(directory.path().join("bar").exists());
    }

    #[test]
    fn fail_to_run_command() {
        let directory = tempdir().unwrap();

        assert_eq!(
            run_build_script(
                directory.path(),
                &format!(
                    "rule fail\n  command = false\nbuild {}: fail\n",
                    directory.path().join("foo").display()
                ),
                "foo",
            )
            .unwrap_err()
            .downcast_ref::<BuildError>(),
            Some(&BuildError::Build)
        );
    }

    #[test]
    fn fail_to_find_target() {
        let directory = tempdir().unwrap();

        assert_eq!(
            run_build_script(
                directory.path(),
                &compile_build_script(directory.path(), &["foo"]),
                "bar",
            )
            .unwrap_err()
            .downcast_ref::<BuildError>(),
            Some(&BuildError::OutputNotFound(
                directory.path().join("bar").display().to_string()
            ))
        );
    }

    #[test]
    fn fail_to_parse_build_script() {
        let directory = tempdir().unwrap();

        assert!(matches!(
            run_build_script(directory.path(), "build\n", "foo")
                .unwrap_err()
                .downcast_ref::<BuildError>(),
            Some(BuildError::Parse(_))
        ));
    }
}
