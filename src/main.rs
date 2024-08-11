use std::{
    fs,
    path::{Path, PathBuf},
};

use tree_sitter::{Node, Parser, Point, TreeCursor};
use walkdir::{DirEntry, WalkDir};

const AS_OPERATOR_ID: u16 = 234;
const COMMENT_OPERATOR_ID: u16 = 410;

#[derive(Debug)]
struct FailureNode {
    grammar_id: u16,
    grammar_name: String,
    start_position: Point,
    end_position: Point,
    file_path: PathBuf,
}

impl Into<FailureNode> for (&Path, Node<'_>) {
    fn into(self) -> FailureNode {
        let (file_path, node) = self;
        FailureNode {
            grammar_name: node.grammar_name().to_owned(),
            file_path: file_path.to_path_buf(),
            grammar_id: node.grammar_id(),
            start_position: node.start_position(),
            end_position: node.end_position(),
        }
    }
}

fn main() {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_dart::language())
        .expect("Could not load Dart grammar");

    let failures: Vec<FailureNode> = WalkDir::new("test_files/comment")
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
            Ok(source) => Some((entry, source)),
            Err(e) => {
                println!("{e}");
                None
            }
        })
        .map(|(entry, source_code)| {
            (
                entry,
                parser.parse(source_code, None).expect("Could not parse"),
            )
        })
        .flat_map(|(entry, tree)| {
            let cursor = tree.walk();
            traverse(entry.path(), cursor, |node| {
                is_as(node.clone()) || is_comment(node.clone())
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

// Inspired by: https://github.com/skmendez/tree-sitter-traversal/blob/main/src/lib.rs
fn traverse<F>(file_path: &Path, mut cursor: TreeCursor, mut callback: F) -> Vec<FailureNode>
where
    F: FnMut(Node) -> bool,
{
    let mut failures: Vec<FailureNode> = Vec::new();
    loop {
        // println!("name: {}, id: {}", cursor.node().grammar_name(), cursor.node().grammar_id());
        if callback(cursor.node()) {
            failures.push((file_path, cursor.node()).into());
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
