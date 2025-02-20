use ast_analyser::failure_finder::{FailureFinder, FailureOutput};
use ast_analyser::cli_arguments::NodeAnalyser;
use clap::Parser;

fn main() -> FailureOutput {
    let args = NodeAnalyser::parse();

    let failure_finder = FailureFinder::default();

    let failures = match args {
        NodeAnalyser::File(args) => failure_finder.analyse_file(args.file_path),
        NodeAnalyser::Directory(args) => failure_finder.analyse_directory(args.directory_path),
        NodeAnalyser::Files(args) => failure_finder.analyse_files(args.file_paths),
    };

    FailureOutput::new(failures)
}
