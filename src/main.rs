use std::{fs, path::Path, process::exit};

use anyhow::{anyhow, Error};

use clap::Parser;
use tree_sitter::{Node, Point, TreeCursor};
use walkdir::WalkDir;

const BANG_OPERATOR_ID: u16 = 64;
const AS_OPERATOR_ID: u16 = 234;
const COMMENT_OPERATOR_ID: u16 = 410;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct CLIArguments {
    #[clap(short = 'd', long = "directory", default_value = "test_files")]
    directory_path: String,
}

#[derive(Debug)]
struct FailureNode {
    grammar_name: String,
    grammar_id: u16,
    start_position: Point,
    end_position: Point,
}

impl TryFrom<(&'_ [u8], Node<'_>)> for FailureNode {
    type Error = Error;

    fn try_from(value: (&'_ [u8], Node<'_>)) -> Result<Self, Self::Error> {
        let (source, node) = value;
        if is_as(node) || is_bang(node) || is_comment(node) {
            return Ok(FailureNode {
                grammar_name: node.grammar_name().to_owned(),
                grammar_id: node.grammar_id(),
                start_position: node.start_position(),
                end_position: node.end_position(),
            });
        }

        let text = node.utf8_text(source)?.to_lowercase();

        if text.contains("todo") {
            return Ok(FailureNode {
                grammar_name: node.grammar_name().to_owned(),
                grammar_id: node.grammar_id(),
                start_position: node.start_position(),
                end_position: node.end_position(),
            });
        }
        Err(anyhow!("The comment does not contain a TODO"))
    }
}

fn main() {
    let args: CLIArguments = CLIArguments::parse();
    let files_directory = Path::new(&args.directory_path);

    if !files_directory.exists() {
        println!(
            "The provided directory does not exist: {:#?}",
            files_directory
        );
        println!("Exiting");
        exit(1)
    }
    let mut parser = TreeParser::new();
    parser
        .set_language(&tree_sitter_dart::language())
        .expect("Could not load Dart grammar");

    let failures: Vec<FailureNode> = WalkDir::new(files_directory)
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
            Ok(source) => Some(source),
            Err(e) => {
                println!("{e}");
                None
            }
        })
        .flat_map(|source_code| {
            let tree = parser.parse(&source_code, None).expect("Could not parse");
            traverse(&source_code, tree.walk(), |node| {
                is_as(node) || is_comment(node) || is_bang(node)
            })
        })
        .collect();

    println!("Failures: {}", failures.len());
    for failure in failures {
        println!("{:#?}", failure)
    }
}

fn is_as(node: Node) -> bool {
    node.grammar_id() == AS_OPERATOR_ID
}

fn is_comment(node: Node) -> bool {
    node.grammar_id() == COMMENT_OPERATOR_ID
}

fn is_bang(node: Node) -> bool {
    node.grammar_id() == BANG_OPERATOR_ID
}

// Inspired by: https://github.com/skmendez/tree-sitter-traversal/blob/main/src/lib.rs
fn traverse<F>(source: &[u8], mut cursor: TreeCursor, mut callback: F) -> Vec<FailureNode>
where
    F: FnMut(Node) -> bool,
{
    let mut failures = Vec::new();
    loop {
        // println!("name: {}, id: {}", cursor.node().grammar_name(), cursor.node().grammar_id());
        let node = cursor.node();
        if callback(node) {
            if let Ok(failure) = FailureNode::try_from((source, node)) {
                failures.push(failure);
            }
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
