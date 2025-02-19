use clap::Parser;
use failure_finder::FailureFinder;
use std::path::PathBuf;
mod failure_finder;

#[derive(Parser, Debug)]
#[command(author, version, about = "Find failures in the provided file", long_about = None)]
struct FileArguments {
    #[clap(
        short = 'f',
        long = "path",
        default_value = "test_files/bang/bang.dart"
    )]
    file_path: PathBuf,
}

#[derive(Parser, Debug)]
#[command(author, version, about = "Find failures in all files in a directory recursively", long_about = None)]
struct DirectoryArguments {
    #[clap(short = 'd', long = "directory", default_value = "test_files")]
    directory_path: PathBuf,
}

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Find failures in multiple files",
    long_about = "Find failures in multiple files. Seperated by commas - no spaces"
)]
struct FilesArguments {
    #[clap(
        short = 'f',
        long = "files",
        value_delimiter = ',',
        default_value = "test_files/bang/bang.dart,test_files/bang/bang_copy.dart"
    )]
    file_paths: Vec<PathBuf>,
}

#[derive(Parser, Debug)]
#[clap(name = "NodeAnalyser ")]
#[command(version, about, long_about = None)]
enum NodeAnalyser {
    File(FileArguments),
    Directory(DirectoryArguments),
    Files(FilesArguments),
}

fn main() {
    let args = NodeAnalyser::parse();

    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_dart::language())
        .expect("Could not load Dart grammar");

    let failures = match args {
        NodeAnalyser::File(args) => FailureFinder::analyse_file(&mut parser, args),
        NodeAnalyser::Directory(args) => FailureFinder::analyse_directory(&mut parser, args),
        NodeAnalyser::Files(args) => FailureFinder::analyse_files(&mut parser, args),
    };

    println!(
        "Failures: {}",
        failures
            .iter()
            .flat_map(|failure_file| &failure_file.failure_nodes)
            .count()
    );

    for failure in failures {
        println!("{:#?}", failure);
    }
}
