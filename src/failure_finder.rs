use std::{
    fs,
    path::{Path, PathBuf},
    process::{exit, ExitCode, Termination},
};

use serde::{Deserialize, Serialize};
use tree_sitter::{Node, Point, TreeCursor};
use walkdir::WalkDir;

const BANG_OPERATOR_ID: u16 = 64;

pub struct FailureFinder {
    parser: tree_sitter::Parser,
}

impl FailureFinder {
    pub fn analyse_file(mut self, file_path: PathBuf) -> Vec<FailureFile> {
        let source_file = match fs::read(&file_path) {
            Ok(source) => SourceFile::new(file_path, source),
            Err(e) => {
                eprintln!("Could not read file_path: {e}");
                exit(1);
            }
        };

        source_file
            .find_failures(&mut self.parser)
            .into_iter()
            .collect()
    }

    pub fn analyse_files(mut self, file_paths: Vec<PathBuf>) -> Vec<FailureFile> {
        file_paths
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
            .flat_map(|source_file| source_file.find_failures(&mut self.parser))
            .collect()
    }

    pub fn analyse_directory(mut self, directory_path: PathBuf) -> Vec<FailureFile> {
        let files_directory = Path::new(&directory_path);
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
            .flat_map(|source_file| source_file.find_failures(&mut self.parser))
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
            parser,
        }
    }
}


#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FailureOutput {
    #[serde(rename = "transgressions")]
    failures: Vec<FailureFile>,
    transgression_count: usize,
    files_with_transgressions: usize,
}

impl FailureOutput {
    pub fn new(failures: Vec<FailureFile>) -> Self {
        let files_with_transgressions = failures.len();
        let transgression_count = failures
            .iter()
            .flat_map(|failure_file| &failure_file.failure_nodes)
            .count();

        Self {
            failures,
            transgression_count,
            files_with_transgressions,
        }
    }
}

impl Termination for FailureOutput {
    fn report(self) -> ExitCode {
        match serde_json::to_string_pretty(&self) {
            Ok(result) => {
                println!("{result}");
                ExitCode::SUCCESS
            }
            Err(e) => {
                println!("Failed to output failures: {:#?}", e);
                ExitCode::FAILURE
            }
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
    fn new(file_path: PathBuf, source: Vec<u8>) -> SourceFile {
        SourceFile { file_path, source }
    }
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

#[derive(Serialize, Deserialize)]
#[serde(remote = "Point")]
struct PointSerde {
    pub row: usize,
    pub column: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FailureNode {
    id: u16,
    name: String,
    #[serde(with = "PointSerde")]
    start_position: Point,
    #[serde(with = "PointSerde")]
    end_position: Point,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
