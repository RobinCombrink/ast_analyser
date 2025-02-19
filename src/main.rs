use clap::Parser;
use std::{
    fs,
    path::{Path, PathBuf},
    process::exit,
};
use tree_sitter::{Node, Point, TreeCursor};
use walkdir::WalkDir;

const BANG_OPERATOR_ID: u16 = 64;
const AS_OPERATOR_ID: u16 = 234;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct CLIArguments {
    #[clap(short = 'd', long = "directory", default_value = "test_files")]
    directory_path: String,
    #[clap(short = 'a', long = "as-operator", default_value = "false")]
    as_operator: bool,
    #[clap(short = 'b', long = "bang-operator", default_value = "true")]
    bang_operator: bool,
}

#[derive(Debug)]
struct FailureNode {
    id: u16,
    name: String,
    start_position: Point,
    end_position: Point,
}

#[derive(Debug)]
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
    let args: CLIArguments = CLIArguments::parse();
    let files_directory = Path::new(&args.directory_path);

    if !files_directory.exists() {
        eprintln!(
            "The provided directory does not exist: {:#?}",
            files_directory
        );
        eprintln!("Exiting");
        exit(1);
    }

    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_dart::language())
        .expect("Could not load Dart grammar");

    let failures: Vec<FailureFile> = WalkDir::new(files_directory)
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
        .map(|source_file| {
            let tree = parser
                .parse(&source_file.source, None)
                .expect("Could not parse");
            let failure_nodes = traverse(tree.walk(), |node| {
                if is_as(node) || is_bang(node) {
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
        })
        .filter_map(|failure_file| failure_file)
        .collect();

    println!("Failures: {}", failures.len());
    for failure in failures {
        println!("{:#?}", failure);
    }
}

fn is_as(node: Node) -> bool {
    node.grammar_id() == AS_OPERATOR_ID
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
