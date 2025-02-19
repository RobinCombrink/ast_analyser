use std::{
    fs,
    path::{Path, PathBuf},
    process::exit,
};

use tree_sitter::{Node, Point, TreeCursor};
use walkdir::WalkDir;

use crate::{DirectoryArguments, FileArguments, FilesArguments};

const BANG_OPERATOR_ID: u16 = 64;

pub struct FailureFinder {
    parser: tree_sitter::Parser,
}

impl FailureFinder {
    pub fn analyse_file(parser: &mut tree_sitter::Parser, args: FileArguments) -> Vec<FailureFile> {
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

    pub fn analyse_files(
        parser: &mut tree_sitter::Parser,
        args: FilesArguments,
    ) -> Vec<FailureFile> {
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

    pub fn analyse_directory(
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
}

impl Default for FailureFinder {
    fn default() -> Self {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_dart::language())
            .expect("Could not load Dart grammar");
        Self {
            parser: Default::default(),
        }
    }
}

trait FindFailureNode {
    fn find_failure(&self) -> Option<FailureNode>;
}

impl FindFailureNode for Node<'_> {
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

#[derive(Debug, Clone)]
pub(crate) struct FailureNode {
    id: u16,
    name: String,
    start_position: Point,
    end_position: Point,
}

#[derive(Debug, Clone)]
pub(crate) struct FailureFile {
    file_path: PathBuf,
    pub(crate) failure_nodes: Vec<FailureNode>,
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
