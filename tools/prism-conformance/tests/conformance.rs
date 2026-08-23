use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

const RUBY_SPEC_ALLOWLIST: &str = "test/prism/ruby-spec-allowlist.txt";

fn repository_path(path: impl AsRef<Path>) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

fn parser() -> tree_sitter::Parser {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_ruby::LANGUAGE.into())
        .expect("failed to load the Ruby grammar");
    parser
}

fn ruby_files(root: &Path) -> Vec<PathBuf> {
    fn collect(directory: &Path, files: &mut Vec<PathBuf>) {
        let mut entries = fs::read_dir(directory)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", directory.display()))
            .collect::<Result<Vec<_>, _>>()
            .unwrap_or_else(|error| panic!("failed to enumerate {}: {error}", directory.display()));
        entries.sort_by_key(std::fs::DirEntry::path);

        for entry in entries {
            let path = entry.path();
            if path.is_dir() {
                collect(&path, files);
            } else if path.extension().is_some_and(|extension| extension == "rb") {
                files.push(path);
            }
        }
    }

    let mut files = Vec::new();
    collect(root, &mut files);
    files
}

fn tree_sitter_errors(parser: &mut tree_sitter::Parser, source: &[u8]) -> Vec<String> {
    let tree = parser
        .parse(source, None)
        .expect("Tree-sitter cancelled the parse");
    let root = tree.root_node();
    if !root.has_error() {
        return Vec::new();
    }

    let mut cursor = root.walk();
    let mut pending = vec![root];
    let mut errors = Vec::new();

    while let Some(node) = pending.pop() {
        if node.is_error() || node.is_missing() {
            let start = node.start_position();
            let end = node.end_position();
            let kind = if node.is_missing() {
                format!("MISSING {}", node.kind())
            } else {
                "ERROR".to_owned()
            };
            errors.push(format!(
                "{} at {}:{}-{}:{}",
                kind,
                start.row + 1,
                start.column + 1,
                end.row + 1,
                end.column + 1
            ));
        }

        cursor.reset(node);
        if cursor.goto_first_child() {
            loop {
                pending.push(cursor.node());
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }
    }

    errors.sort();
    errors
}

fn prism_diagnostics(source: &[u8]) -> Vec<String> {
    let result = prism::parse(source);

    result
        .errors()
        .map(|diagnostic| {
            let location = diagnostic.location();
            format!(
                "{} at byte {}-{}",
                diagnostic.message(),
                location.start_offset(),
                location.end_offset()
            )
        })
        .collect()
}

fn load_allowlist(path: &Path) -> BTreeMap<String, String> {
    let contents = fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
    let mut entries = BTreeMap::new();

    for (index, line) in contents.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let (fixture, reason) = line.split_once('\t').unwrap_or_else(|| {
            panic!(
                "{}:{}: expected a tab-separated path and reason",
                path.display(),
                index + 1
            )
        });
        assert!(
            !reason.trim().is_empty(),
            "{}:{}: allowlist reason cannot be empty",
            path.display(),
            index + 1
        );
        assert!(
            entries
                .insert(fixture.to_owned(), reason.trim().to_owned())
                .is_none(),
            "{}:{}: duplicate allowlist entry for {fixture}",
            path.display(),
            index + 1
        );
    }

    entries
}

#[test]
#[ignore = "requires PRISM_RUBY_SPEC_ROOT"]
fn ruby_spec_matches_prism() {
    let configured_root = PathBuf::from(
        std::env::var_os("PRISM_RUBY_SPEC_ROOT")
            .expect("PRISM_RUBY_SPEC_ROOT must point to a pinned ruby/spec checkout"),
    );
    let root = if configured_root.is_absolute() {
        configured_root
    } else {
        repository_path(configured_root)
    };
    let allowlist = load_allowlist(&repository_path(RUBY_SPEC_ALLOWLIST));
    let mut consumed_allowlist = BTreeSet::new();
    let mut parser = parser();
    let mut prism_valid = 0;
    let mut prism_invalid = 0;
    let mut expected_mismatches = 0;
    let mut unexpected_mismatches = Vec::new();

    for path in ruby_files(&root) {
        let source = fs::read(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
        if !prism_diagnostics(&source).is_empty() {
            prism_invalid += 1;
            continue;
        }
        prism_valid += 1;

        let errors = tree_sitter_errors(&mut parser, &source);
        if errors.is_empty() {
            continue;
        }

        let relative = path
            .strip_prefix(&root)
            .expect("ruby/spec path escaped its root")
            .to_string_lossy()
            .replace('\\', "/");
        if allowlist.contains_key(&relative) {
            consumed_allowlist.insert(relative);
            expected_mismatches += 1;
        } else {
            unexpected_mismatches.push(format!(
                "{relative}:\n  {}",
                errors.into_iter().take(10).collect::<Vec<_>>().join("\n  ")
            ));
        }
    }

    let stale_allowlist = allowlist
        .keys()
        .filter(|path| !consumed_allowlist.contains(*path))
        .collect::<Vec<_>>();

    println!(
        "Prism conformance: {prism_valid} valid files, {prism_invalid} invalid files, \
         {expected_mismatches} expected mismatches, {} unexpected mismatches",
        unexpected_mismatches.len()
    );

    assert!(
        unexpected_mismatches.is_empty(),
        "Tree-sitter rejected Prism-valid ruby/spec files:\n\n{}",
        unexpected_mismatches.join("\n\n")
    );
    assert!(
        stale_allowlist.is_empty(),
        "ruby/spec allowlist entries no longer fail:\n  {}",
        stale_allowlist
            .into_iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}
