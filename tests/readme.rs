//! The code shown in the README and on the website is not compiled as is,
//! as it leaves out what is not of interest. To keep it from drifting away from code that works,
//! every line of it has to be found, in order, in one of the examples,
//! or in the code of the README that is compiled as a doctest.

use std::fs;
use std::path::Path;

/// Code without whitespace and commas, which `rustfmt` adds and removes as it sees fit.
fn squash(code: &str) -> String {
    code.chars()
        .filter(|c| !c.is_whitespace() && *c != ',')
        .collect()
}

/// The code blocks of the README with the given info string, and the line they start at.
fn blocks(readme: &str, info: &str) -> Vec<(usize, String)> {
    let mut blocks = Vec::new();
    let mut lines = readme.lines().enumerate();
    while let Some((number, line)) = lines.next() {
        if line.trim() == format!("```{info}") {
            let code: Vec<&str> = lines
                .by_ref()
                .map(|(_, line)| line)
                .take_while(|line| line.trim() != "```")
                .collect();
            blocks.push((number + 1, code.join("\n")));
        }
    }
    blocks
}

/// The first line of the code that is not found in the source, after the lines before it.
fn missing<'a>(code: &'a str, source: &str) -> Option<&'a str> {
    let mut rest = source;
    for line in code.lines().filter(|line| !line.trim().is_empty()) {
        match rest.find(&squash(line)) {
            Some(at) => rest = &rest[at + squash(line).len()..],
            None => return Some(line),
        }
    }
    None
}

#[test]
fn code_of_the_readme_is_tested() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let readme = fs::read_to_string(root.join("README.md")).unwrap();

    let mut sources: Vec<(String, String)> = fs::read_dir(root.join("examples"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "rs"))
        .map(|path| {
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            (name, squash(&fs::read_to_string(&path).unwrap()))
        })
        .collect();
    for (line, code) in blocks(&readme, "rust") {
        sources.push((format!("README.md:{line}"), squash(&code)));
    }
    assert!(sources.len() > 6, "examples not found");

    let mut snippets: Vec<(String, String)> = blocks(&readme, "rust,ignore")
        .into_iter()
        .map(|(line, code)| (format!("README.md:{line}"), code))
        .collect();
    assert!(snippets.len() >= 8, "code of the README not found");
    let website = fs::read_to_string(root.join("www/index.html")).unwrap();
    for (n, code) in website.split("<pre><code>").skip(1).enumerate() {
        let code = code.split("</code></pre>").next().unwrap();
        let code = code
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&amp;", "&");
        snippets.push((format!("www/index.html, code block {}", n + 1), code));
    }
    assert!(snippets.len() >= 10, "code of the website not found");
    for (line, code) in snippets {
        let closest = sources
            .iter()
            .map(|(name, source)| (name, missing(&code, source)))
            .min_by_key(|(_, missing)| {
                missing.map_or(0, |line| code.len() - code.find(line).unwrap_or(0))
            })
            .unwrap();
        if let (name, Some(missing)) = closest {
            panic!(
                "{line}: not part of any example, the closest being {name}, \
                 which has no:\n{missing}"
            );
        }
    }
}

#[test]
fn squashed_code_ignores_layout() {
    assert_eq!(
        squash("Foo { a: 1,\n    b: 2,\n}"),
        squash("Foo { a: 1, b: 2 }")
    );
    assert_eq!(squash("f(\n    a,\n    b,\n)"), squash("f(a, b)"));
    assert_ne!(squash("a.b()"), squash("a.c()"));
    assert_eq!(missing("b\n\n  c", "abcd"), None);
    assert_eq!(missing("c\nb", "abcd"), Some("b"));
    assert_eq!(missing("a\ne", "abcd"), Some("e"));
    assert_eq!(
        blocks("x\n```rust\na\nb\n```\n```sh\nc\n```", "rust"),
        [(2, "a\nb".to_owned())]
    );
}
