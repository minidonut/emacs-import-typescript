use find_up::find_up;
use globset::Glob;
use ignore::Walk;
use package_json::read_package_json;
use pathdiff::diff_paths;
use regex::Regex;
use std::process;
use std::{env, path::PathBuf};

use crate::package_json::{get_dependencies, ExcludePattern};

mod find_up;
mod package_json;

fn main() {
    let (package_json_path, project_path) = get_paths();

    let project_files: Vec<String> = get_project_files(&project_path);
    let project_dependencies = get_project_dependencies(&package_json_path);

    let candidates: String = project_dependencies
        .iter()
        .chain(project_files.iter())
        .map(|s| format!("\"{}\"", s))
        .collect::<Vec<_>>()
        .join(" ");

    println!(
        r#"
(ivy-read "import {{}} from "
  (list {})
  :action (lambda (x) x))
"#,
        candidates,
    )
}

fn get_project_files(project_path: &PathBuf) -> Vec<String> {
    let glob = Glob::new("*.{ts,tsx}").unwrap().compile_matcher();
    let current_dir = env::current_dir().unwrap();

    Walk::new(project_path)
        .filter_map(|result| result.ok())
        .filter(|entry| glob.is_match(entry.path()))
        .filter_map(|entry| {
            diff_paths(entry.path(), &current_dir).map(|rel_path| {
                let mut path_str = rel_path.display().to_string();
                path_str = path_str
                    .replace('\\', "/")
                    .replace("/index.ts", "")
                    .replace("/index.tsx", "");

                // Strip the file extension (.ts or .tsx)
                path_str = path_str
                    .trim_end_matches(".ts")
                    .trim_end_matches(".tsx")
                    .to_string();

                // Append "./" if the path_str does not start with "."
                if !path_str.starts_with('.') {
                    path_str = format!("./{}", path_str);
                }

                path_str
            })
        })
        .collect()
}

fn get_project_dependencies(package_json_path: &PathBuf) -> Vec<String> {
    let package_json = read_package_json(package_json_path.as_path());

    let dependencies: Vec<String> = get_dependencies("dependencies", &package_json, &[])
        .iter()
        .map(|&s| s.to_string())
        .collect();

    let mut dev_dependencies = get_dependencies(
        "devDependencies",
        &package_json,
        &[
            ExcludePattern::Plain("jest"),
            ExcludePattern::Plain("ts-jest"),
            ExcludePattern::Plain("ts-node"),
            ExcludePattern::Regex(Regex::new(r"eslint").unwrap()),
            ExcludePattern::Regex(Regex::new(r"^@types/").unwrap()),
        ],
    )
    .iter()
    .map(|&s| s.to_string())
    .collect();

    let mut all_dependencies = dependencies;
    all_dependencies.append(&mut dev_dependencies);
    all_dependencies
}

pub fn get_paths() -> (PathBuf, PathBuf) {
    let package_json_path = match find_up("package.json").unwrap() {
        Some(path) => path,
        None => {
            println!("(message \"package.json does not exist\")");
            process::exit(0)
        }
    };

    let project_path = match package_json_path.parent() {
        Some(path) => path.to_path_buf(),
        None => {
            println!("(message \"Cannot find project directory\")");
            process::exit(0)
        }
    };

    (package_json_path, project_path)
}
