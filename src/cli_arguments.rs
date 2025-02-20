use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about = "Find failures in the provided file", long_about = None)]
pub struct FileArguments {
    #[clap(
        short = 'f',
        long = "path",
        default_value = "test_files/bang/bang.dart"
    )]
    pub file_path: PathBuf,
}

#[derive(Parser, Debug)]
#[command(author, version, about = "Find failures in all files in a directory recursively", long_about = None)]
pub struct DirectoryArguments {
    #[clap(short = 'd', long = "directory", default_value = "test_files")]
    pub directory_path: PathBuf,
}

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Find failures in multiple files",
    long_about = "Find failures in multiple files. Seperated by commas - no spaces"
)]
pub struct FilesArguments {
    #[clap(
        short = 'f',
        long = "files",
        value_delimiter = ',',
        default_value = "test_files/bang/bang.dart,test_files/bang/bang_copy.dart"
    )]
    pub file_paths: Vec<PathBuf>,
}

#[derive(Parser, Debug)]
#[clap(name = "NodeAnalyser")]
#[command(version, about, long_about = None)]
pub enum NodeAnalyser {
    File(FileArguments),
    Directory(DirectoryArguments),
    Files(FilesArguments),
}
