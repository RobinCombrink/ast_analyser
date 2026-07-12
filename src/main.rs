use anyhow::{anyhow, Result};
use ast_analyser::cli_arguments::NodeAnalyser;
use ast_analyser::failure_finder::{FailureFinder, FailureOutput};
use clap::Parser;

fn main() -> Result<FailureOutput> {
    let args = NodeAnalyser::parse();

    let mut failure_finder = FailureFinder::default();

    let failures = match args {
        NodeAnalyser::File(args) => Ok(vec![failure_finder.analyse_file(args.file_path)?]),
        NodeAnalyser::Files(args) => failure_finder.analyse_files(args.file_paths),
        NodeAnalyser::Directory(args) => failure_finder.analyse_directory(args.directory_path),
    };

    match failures {
        Ok(failures) => Ok(FailureOutput::new(
            failures.into_iter().flatten().collect(),
        )),
        Err(err) => {
            Err(anyhow!("Something went wrong finding transgressions:\n{:?}", err))
        }
    }
}
