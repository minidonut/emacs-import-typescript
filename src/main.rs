use find_up::find_up;
use globset::Glob;
use ignore::Walk;
use serde_json::Value;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::process;

mod find_up;

fn main() {
    let package_json_path = match find_up("package.json").unwrap() {
        Some(path) => path,
        None => {
            println!("(message \"package.json does not exist\")");
            process::exit(0)
        }
    };

    let project_path = match package_json_path.parent() {
        Some(path) => path,
        None => {
            println!("(message \"Cannot find project directory\")");
            process::exit(0)
        }
    };

    let glob = Glob::new("*.ts").unwrap().compile_matcher();

    for result in Walk::new(project_path.to_str().unwrap()) {
        if let Ok(entry) = result {
            if glob.is_match(entry.path()) {
                println!("{}", entry.path().display())
            }
        }
    }

    let package_json = read_package_json(package_json_path.as_path());

    println!("package.json {}", package_json["dependencies"]);

    println!(
        "(message \"package.json path is {}\")",
        package_json_path.to_str().unwrap()
    );
}

fn read_package_json(path: &Path) -> Value {
    let mut file = File::open(path).expect("Unable to open the file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Unable to read the file");

    let json_value: Value = serde_json::from_str(&contents).expect("Failed to parse JSON");
    json_value
}
