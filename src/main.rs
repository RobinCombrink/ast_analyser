use clap::Parser;
use std::{
    fs,
    path::{Path, PathBuf},
    process::exit,
};
use tree_sitter::{Node, Point, TreeCursor};
use walkdir::WalkDir;

const BANG_OPERATOR_ID: u16 = 64;

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

#[derive(Debug, Clone)]
struct FailureNode {
    id: u16,
    name: String,
    start_position: Point,
    end_position: Point,
}

#[derive(Debug, Clone)]
struct FailureFile {
    file_path: PathBuf,
    failure_nodes: Vec<FailureNode>,
}

impl From<&Node<'_>> for FailureNode {
    fn from(value: &Node) -> Self {
        FailureNode {
            id: value.grammar_id(),
            name: value.grammar_name().to_owned(),
            start_position: value.start_position(),
            end_position: value.end_position(),
        }
    }
}

trait FailureFinder {
    fn find_failure(&self) -> Option<FailureNode>;
}

impl FailureFinder for Node<'_> {
    fn find_failure(&self) -> Option<FailureNode> {
            if is_bang(self) {
                Some(FailureNode::from(self))
            } else {
                None
            }
    }
}


struct SourceFile {
    file_path: PathBuf,
    source: Vec<u8>,
}

impl SourceFile {
    fn find_failures(self, parser: &mut tree_sitter::Parser) -> Option<FailureFile> {
        let tree = parser.parse(&self.source, None).expect(&format!(
            "Could not parse source file: {:#?}",
            self.file_path
        ));

        let failure_nodes = traverse(tree.walk(), |node| node.find_failure());

        if !failure_nodes.is_empty() {
            Some(FailureFile {
                file_path: self.file_path,
                failure_nodes,
            })
        } else {
            None
        }
    }
}

fn main() {
    let args = NodeAnalyser::parse();

    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_dart::language())
        .expect("Could not load Dart grammar");

    let failures = match args {
        NodeAnalyser::File(args) => analyse_file(&mut parser, args),
        NodeAnalyser::Directory(args) => analyse_directory(&mut parser, args),
        NodeAnalyser::Files(args) => analyse_files(&mut parser, args),
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

fn analyse_file(parser: &mut tree_sitter::Parser, args: FileArguments) -> Vec<FailureFile> {
    let source_file = match fs::read(&args.file_path) {
        Ok(source) => SourceFile {
            file_path: args.file_path,
            source,
        },
        Err(e) => {
            eprintln!("Could not read file_path: {e}");
            exit(1);
        }
    };

    source_file.find_failures(parser).into_iter().collect()
}

fn analyse_files(parser: &mut tree_sitter::Parser, args: FilesArguments) -> Vec<FailureFile> {
    args.file_paths
        .iter()
        .filter_map(|file_path| match fs::read(file_path) {
            Ok(source) => Some(SourceFile {
                file_path: PathBuf::from(file_path),
                source,
            }),
            Err(e) => {
                println!("Could not read file at: {:#?}\n{e}", file_path);
                None
            }
        })
        .flat_map(|source_file| source_file.find_failures(parser))
        .collect()
}

fn analyse_directory(
    parser: &mut tree_sitter::Parser,
    args: DirectoryArguments,
) -> Vec<FailureFile> {
    let files_directory = Path::new(&args.directory_path);
    if !files_directory.exists() {
        eprintln!(
            "The provided directory does not exist: {:#?}",
            files_directory
        );
        exit(1);
    }

    WalkDir::new(files_directory)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| !entry.file_type().is_dir())
        .filter_map(|entry| match fs::read(entry.path()) {
            Ok(source) => Some(SourceFile {
                file_path: entry.into_path(),
                source,
            }),
            Err(e) => {
                println!("{e}");
                None
            }
        })
        .flat_map(|source_file| source_file.find_failures(parser))
        .collect()
}

fn is_bang(node: &Node) -> bool {
    node.grammar_id() == BANG_OPERATOR_ID
}

// Inspired by: https://github.com/skmendez/tree-sitter-traversal/blob/main/src/lib.rs
fn traverse<F>(mut cursor: TreeCursor, mut callback: F) -> Vec<FailureNode>
where
    F: FnMut(Node) -> Option<FailureNode>,
{
    let mut failures = Vec::new();
    loop {
        // println!("name: {}, id: {}", cursor.node().grammar_name(), cursor.node().grammar_id());
        let node = cursor.node();
        if let Some(failure) = callback(node) {
            failures.push(failure);
        }

        if cursor.goto_first_child() {
            continue;
        }

        if cursor.goto_next_sibling() {
            continue;
        }

        loop {
            if !cursor.goto_parent() {
                return failures;
            }

            if cursor.goto_next_sibling() {
                break;
            }
        }
    }
}
