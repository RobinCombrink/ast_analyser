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
    #[clap(short = 'f', long = "path", default_value = "test_files/bang/bang.dart")]
    file_path: PathBuf,
}

#[derive(Parser, Debug)]
#[command(author, version, about = "Find failures in all files in a directory recursively", long_about = None)]
struct DirectoryArguments {
    #[clap(short = 'd', long = "directory", default_value = "test_files")]
    directory_path: PathBuf,
}

#[derive(Parser, Debug)]
#[command(author, version, about = "Find failures in multiple files", long_about = "Find failures in multiple files. Seperated by commas - no spaces")]
struct FilesArguments {
    #[clap(
        short = 'd',
        long = "directory",
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

impl From<Node<'_>> for FailureNode {
    fn from(value: Node) -> Self {
        FailureNode {
            id: value.grammar_id(),
            name: value.grammar_name().to_owned(),
            start_position: value.start_position(),
            end_position: value.end_position(),
        }
    }
}

struct SourceFile {
    file_path: PathBuf,
    source: Vec<u8>,
}

fn main() {
    let args = NodeAnalyser::parse();

    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_dart::language())
        .expect("Could not load Dart grammar");

    let failures: Vec<FailureFile> = match args {
        NodeAnalyser::File(args) => analyse_file(parser, args),
        NodeAnalyser::Directory(args) => analyse_directory(parser, args),
        NodeAnalyser::Files(args) => analyse_files(parser, args),
    };

    println!("Failures: {}", failures.clone().into_iter().flat_map(|failure_file| failure_file.failure_nodes).collect::<Vec<FailureNode>>().len());
    for failure in failures {
        println!("{:#?}", failure);
    }
}

fn analyse_file(mut parser: tree_sitter::Parser, args: FileArguments) -> Vec<FailureFile> {
    let source_file = match fs::read(&args.file_path) {
        Ok(source) => SourceFile {
            file_path: args.file_path,
            source,
        },
        Err(e) => {
            eprintln!("Could not read file_path: {e}");
            eprintln!("Exiting");
            exit(1);
        }
    };

    let failure_file = find_failures(&mut parser, source_file);
    match failure_file {
        Some(failure_file) => vec![failure_file],
        None => vec![],
    }
}

fn analyse_files(mut parser: tree_sitter::Parser, args: FilesArguments) -> Vec<FailureFile> {
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
        .map(move |source_file| find_failures(&mut parser, source_file))
        .filter_map(|failure_file| failure_file)
        .collect()
}

fn analyse_directory(
    mut parser: tree_sitter::Parser,
    args: DirectoryArguments,
) -> Vec<FailureFile> {
    let files_directory = Path::new(&args.directory_path);
    if !files_directory.exists() {
        eprintln!(
            "The provided directory does not exist: {:#?}",
            files_directory
        );
        eprintln!("Exiting");
        exit(1);
    }

    WalkDir::new(files_directory)
        .into_iter()
        .filter_map(|entry| match entry {
            Ok(entry) => Some(entry),
            Err(e) => {
                println!("{e}");
                None
            }
        })
        .filter(|entry| !entry.file_type().is_dir())
        .filter_map(|entry| match fs::read(entry.path()) {
            Ok(source) => Some(SourceFile {
                file_path: entry.into_path().to_path_buf(),
                source,
            }),
            Err(e) => {
                println!("{e}");
                None
            }
        })
        .map(move |source_file| find_failures(&mut parser, source_file))
        .filter_map(|failure_file| failure_file)
        .collect()
}

fn find_failures(parser: &mut tree_sitter::Parser, source_file: SourceFile) -> Option<FailureFile> {
    let tree = parser
        .parse(&source_file.source, None)
        .expect("Could not parse");
    let failure_nodes = traverse(tree.walk(), |node| {
        if is_bang(node) {
            Some(FailureNode::from(node))
        } else {
            None
        }
    });
    if failure_nodes.len() > 0 {
        Some(FailureFile {
            file_path: source_file.file_path,
            failure_nodes,
        })
    } else {
        None
    }
}

fn is_bang(node: Node) -> bool {
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
