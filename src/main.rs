use anyhow::{anyhow, Context, Result};
use ast_analyser::cli_arguments::NodeAnalyser;
use ast_analyser::failure_finder::{FailureFinder, FailureOutput};
use clap::Parser;

fn main() -> Result<FailureOutput> {
    let args = NodeAnalyser::parse();

    let mut failure_finder = FailureFinder::default();

    let failures = match args {
        NodeAnalyser::File(args) => {
            let analysis = failure_finder.analyse_file(args.file_path);
            match analysis {
                Ok(analysis) => Ok(vec![analysis]),
                Err(err) => Err(err),
            }
        }
        NodeAnalyser::Directory(args) => failure_finder.analyse_directory(args.directory_path),
        NodeAnalyser::Files(args) => failure_finder.analyse_files(args.file_paths),
    };

    match failures {
        Ok(failures) => Ok(FailureOutput::new(
            failures.into_iter().filter_map(|failure| failure).collect(),
        )),
        Err(err) => {
            Err(anyhow!("Something went wrong finding transgressions")).with_context(|| err)
        }
    }
}
