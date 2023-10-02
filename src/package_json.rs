use regex::Regex;
use serde_json::Value;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

// Define the enum for exclusion patterns
pub enum ExcludePattern<'a> {
    Plain(&'a str),
    Regex(Regex),
}

pub fn read_package_json(path: &Path) -> Value {
    let mut file = File::open(path).expect("Unable to open the file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Unable to read the file");

    let json_value: Value = serde_json::from_str(&contents).expect("Failed to parse JSON");
    json_value
}

pub fn get_dependencies<'a>(
    key: &str,
    package_json: &'a Value,
    excludes: &[ExcludePattern],
) -> Vec<&'a str> {
    package_json
        .get(key)
        .and_then(Value::as_object)
        .map(|deps| {
            deps.keys()
                .filter(|&key| !should_exclude(key, excludes))
                .map(|key| key.as_str())
                .collect()
        })
        .unwrap_or_else(Vec::new)
}

fn should_exclude(key: &str, excludes: &[ExcludePattern]) -> bool {
    excludes.iter().any(|exclude| match exclude {
        ExcludePattern::Plain(s) => s == &key,
        ExcludePattern::Regex(r) => r.is_match(key),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_with_excludes_and_regex() {
        let data = serde_json::json!({
            "dependencies": {
                "serde": "1.0",
                "serde_json": "1.0",
                "typescript": "4.0",
                "jest": "27.0",
                "@types/jest": "27.0",
                "@types/react": "17.0",
                "rocket": "0.5"
            }
        });

        let excludes = [
            ExcludePattern::Plain("typescript"),
            ExcludePattern::Regex(Regex::new(r"^@types/").unwrap()),
        ];
        let mut deps = get_dependencies("dependencies", &data, &excludes);
        let mut expected = vec!["serde", "serde_json", "jest", "rocket"];

        deps.sort();
        expected.sort();

        assert_eq!(deps, expected);
    }
}
