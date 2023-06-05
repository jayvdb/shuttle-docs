use std::io::Write;
use std::path::{Path, PathBuf};

use slugify::slugify;

/// Find directories containing a `Cargo.toml`.
fn rust_packages(examples_dir: &Path) -> Vec<PathBuf> {
    let paths = glob::glob(examples_dir.join("**/Cargo.toml").to_str().unwrap())
        .expect("Unable to read directory");
    paths
        .map(|x| {
            // Remove `/Cargo.toml`
            let mut y = x.unwrap();
            y.pop();
            PathBuf::from(y.strip_prefix(examples_dir).unwrap())
        })
        .filter(|x| !x.starts_with("fullstack-templates"))
        .collect()
}

fn slugify_snippet_name(example_package_path: String) -> String {
    slugify!(&example_package_path) + ".mdx"
}

/// Convert the file contents into a markdown code block.
fn generate_snippet(file_path: String, language: String, display_name: String) -> String {
    let contents = std::fs::read_to_string(file_path).unwrap();
    format!("```{} {}\n{}\n```\n", language, display_name, contents)
}

/// Write `contents` to `filename`
fn write_snippet(filename: &PathBuf, contents: String) {
    let mut f = std::fs::File::create(filename).unwrap();
    f.write_all(contents.as_bytes()).unwrap();
}

/// Convert a Rust package directory into a set of snippets
fn create_snippets(examples_dir: &Path, example_package_path: PathBuf, snippet_dir: &Path) {
    let mut example_dir = PathBuf::from(examples_dir);
    example_dir.push(example_package_path.clone());
    let files =
        glob::glob(example_dir.join("**/*").to_str().unwrap()).expect("Unable to read directory");
    // Ignore directories and .ignore files
    let files: Vec<PathBuf> = files
        .filter(|x| !x.as_ref().unwrap().metadata().unwrap().is_dir())
        .filter(|x| {
            ![".gitignore", ".ignore"]
                .contains(&x.as_ref().unwrap().file_name().unwrap().to_str().unwrap())
        })
        .map(|x| PathBuf::from(x.unwrap().strip_prefix(example_dir.clone()).unwrap()))
        .collect();

    for file_path in files {
        let relative_file_path =
            format!("{}/{}", example_package_path.display(), file_path.display());
        let snippet_name = slugify_snippet_name(relative_file_path.clone());
        // Some filenames are foo.ext.example, and .example needs to be stripped
        let language_detect_file_name = match file_path.extension().unwrap() == "example" {
            true => PathBuf::from(file_path.clone().file_stem().unwrap()),
            false => file_path.clone(),
        };
        let language = detect_lang::from_path(language_detect_file_name)
            .unwrap_or_else(|| panic!("language detection of {:?} failed", file_path.clone()))
            .to_string();
        let complete_file_path = format!("{}/{}", example_dir.display(), file_path.display());
        let snippet = generate_snippet(
            complete_file_path.clone(),
            language,
            file_path.display().to_string(),
        );

        let mut snippet_filename = snippet_dir.to_path_buf();
        snippet_filename.push(snippet_name);
        write_snippet(&snippet_filename, snippet);
    }
}

fn main() {
    let mut cli_args = std::env::args();
    let examples_dir = cli_args.nth(1).expect("Please provide path to examples");
    let examples_dir = Path::new(&examples_dir);
    if !examples_dir.exists() {
        eprintln!("{} does not exist", examples_dir.display());
    }
    let example_packages = rust_packages(examples_dir);
    let mut snippets_dir = std::env::current_dir().unwrap();
    snippets_dir.push("..");
    snippets_dir.push("_snippets");
    if !snippets_dir.exists() {
        std::fs::create_dir(snippets_dir.clone()).unwrap();
    }
    for example in example_packages {
        create_snippets(examples_dir, example, &snippets_dir);
    }
}
