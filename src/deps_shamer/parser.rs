//! Parsers for various dependency file formats.
//!
//! Supports Cargo.toml, package.json, go.mod, and requirements.txt.

use super::types::{DepFile, DepSource, Dependency, Ecosystem};
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

type DepParser<'a> = (&'a str, fn(&Path) -> Result<DepFile>);

/// Detect and parse dependency files in the given directory.
pub fn detect_and_parse(project_path: &Path) -> Result<Vec<DepFile>> {
    let mut results = Vec::new();

    let candidates: &[DepParser] = &[
        ("Cargo.toml", parse_cargo_toml),
        ("package.json", parse_package_json),
        ("go.mod", parse_go_mod),
        ("requirements.txt", parse_requirements_txt),
        ("pyproject.toml", parse_pyproject_toml),
    ];

    for (filename, parser) in candidates {
        let dep_path = project_path.join(filename);
        if dep_path.exists() {
            match parser(&dep_path) {
                Ok(dep_file) => results.push(dep_file),
                Err(e) => {
                    eprintln!("Warning: failed to parse {}: {}", filename, e);
                }
            }
        }
    }

    Ok(results)
}

/// Parse a Cargo.toml file.
fn parse_cargo_toml(path: &Path) -> Result<DepFile> {
    let content =
        fs::read_to_string(path).with_context(|| format!("Failed to read {}", path.display()))?;
    let doc: toml::Value = content
        .parse()
        .with_context(|| format!("Failed to parse TOML from {}", path.display()))?;

    let mut dependencies = Vec::new();

    // Parse [dependencies]
    if let Some(deps) = doc.get("dependencies").and_then(|v| v.as_table()) {
        for (name, value) in deps {
            let dep = parse_cargo_dep(name, value, false, false);
            dependencies.push(dep);
        }
    }

    // Parse [dev-dependencies]
    if let Some(deps) = doc.get("dev-dependencies").and_then(|v| v.as_table()) {
        for (name, value) in deps {
            let dep = parse_cargo_dep(name, value, true, false);
            dependencies.push(dep);
        }
    }

    // Parse [build-dependencies]
    if let Some(deps) = doc.get("build-dependencies").and_then(|v| v.as_table()) {
        for (name, value) in deps {
            let dep = parse_cargo_dep(name, value, false, false);
            dependencies.push(dep);
        }
    }

    Ok(DepFile {
        path: path.to_string_lossy().to_string(),
        ecosystem: Ecosystem::Rust,
        dependencies,
    })
}

fn parse_cargo_dep(name: &str, value: &toml::Value, is_dev: bool, is_optional: bool) -> Dependency {
    match value {
        // Simple version string: serde = "1.0"
        toml::Value::String(version) => Dependency {
            name: name.to_string(),
            version: version.clone(),
            source: DepSource::Registry,
            is_dev,
            is_optional,
        },
        // Table form: serde = { version = "1.0", features = [...] }
        toml::Value::Table(table) => {
            let version = table
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("*")
                .to_string();

            let source = if let Some(git_url) = table.get("git").and_then(|v| v.as_str()) {
                DepSource::Git {
                    url: git_url.to_string(),
                }
            } else if let Some(p) = table.get("path").and_then(|v| v.as_str()) {
                DepSource::Path {
                    path: p.to_string(),
                }
            } else {
                DepSource::Registry
            };

            let optional = table
                .get("optional")
                .and_then(|v| v.as_bool())
                .unwrap_or(is_optional);

            Dependency {
                name: name.to_string(),
                version,
                source,
                is_dev,
                is_optional: optional,
            }
        }
        _ => Dependency {
            name: name.to_string(),
            version: "*".to_string(),
            source: DepSource::Unknown,
            is_dev,
            is_optional,
        },
    }
}

/// Parse a package.json file.
fn parse_package_json(path: &Path) -> Result<DepFile> {
    let content =
        fs::read_to_string(path).with_context(|| format!("Failed to read {}", path.display()))?;
    let doc: serde_json::Value = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse JSON from {}", path.display()))?;

    let mut dependencies = Vec::new();

    // Parse "dependencies"
    if let Some(deps) = doc.get("dependencies").and_then(|v| v.as_object()) {
        for (name, version_val) in deps {
            let version = version_val.as_str().unwrap_or("*").to_string();
            let source = classify_npm_source(&version);
            dependencies.push(Dependency {
                name: name.clone(),
                version,
                source,
                is_dev: false,
                is_optional: false,
            });
        }
    }

    // Parse "devDependencies"
    if let Some(deps) = doc.get("devDependencies").and_then(|v| v.as_object()) {
        for (name, version_val) in deps {
            let version = version_val.as_str().unwrap_or("*").to_string();
            let source = classify_npm_source(&version);
            dependencies.push(Dependency {
                name: name.clone(),
                version,
                source,
                is_dev: true,
                is_optional: false,
            });
        }
    }

    // Parse "optionalDependencies"
    if let Some(deps) = doc.get("optionalDependencies").and_then(|v| v.as_object()) {
        for (name, version_val) in deps {
            let version = version_val.as_str().unwrap_or("*").to_string();
            dependencies.push(Dependency {
                name: name.clone(),
                version,
                source: DepSource::Registry,
                is_dev: false,
                is_optional: true,
            });
        }
    }

    Ok(DepFile {
        path: path.to_string_lossy().to_string(),
        ecosystem: Ecosystem::Node,
        dependencies,
    })
}

fn classify_npm_source(version: &str) -> DepSource {
    if version.starts_with("git+")
        || version.starts_with("github:")
        || version.starts_with("git://")
    {
        DepSource::Git {
            url: version.to_string(),
        }
    } else if version.starts_with("file:") || version.starts_with("link:") {
        DepSource::Path {
            path: version
                .trim_start_matches("file:")
                .trim_start_matches("link:")
                .to_string(),
        }
    } else {
        DepSource::Registry
    }
}

/// Parse a go.mod file.
fn parse_go_mod(path: &Path) -> Result<DepFile> {
    let content =
        fs::read_to_string(path).with_context(|| format!("Failed to read {}", path.display()))?;

    let mut dependencies = Vec::new();
    let mut in_require_block = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("require (") {
            in_require_block = true;
            continue;
        }

        if in_require_block && trimmed == ")" {
            in_require_block = false;
            continue;
        }

        if in_require_block || trimmed.starts_with("require ") {
            let line_content = if trimmed.starts_with("require ") {
                trimmed.strip_prefix("require ").unwrap_or(trimmed)
            } else {
                trimmed
            };

            let parts: Vec<&str> = line_content.split_whitespace().collect();
            if parts.len() >= 2 {
                let name = parts[0].to_string();
                let version = parts[1].to_string();

                // Check if it's indirect (comment at end)
                let is_indirect = trimmed.contains("// indirect");

                dependencies.push(Dependency {
                    name,
                    version,
                    source: DepSource::Registry,
                    is_dev: false,
                    is_optional: is_indirect,
                });
            }
        }
    }

    Ok(DepFile {
        path: path.to_string_lossy().to_string(),
        ecosystem: Ecosystem::Go,
        dependencies,
    })
}

/// Parse a requirements.txt file (Python).
fn parse_requirements_txt(path: &Path) -> Result<DepFile> {
    let content =
        fs::read_to_string(path).with_context(|| format!("Failed to read {}", path.display()))?;

    let mut dependencies = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip empty lines, comments, and options
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('-') {
            continue;
        }

        // Handle lines like: package>=1.0, package==1.0, package~=1.0, package!=1.0, package>1.0
        let (name, version) =
            if let Some(pos) = trimmed.find(|c: char| ['>', '<', '=', '!', '~'].contains(&c)) {
                (&trimmed[..pos], &trimmed[pos..])
            } else if let Some(pos) = trimmed.find('[') {
                // Handle extras: package[extra]>=1.0
                (&trimmed[..pos], "*")
            } else {
                (trimmed, "*")
            };

        // Handle git URLs
        let source = if trimmed.starts_with("git+") || trimmed.contains("git://") {
            DepSource::Git {
                url: trimmed.to_string(),
            }
        } else if trimmed.starts_with("./")
            || trimmed.starts_with("../")
            || trimmed.starts_with("/")
        {
            DepSource::Path {
                path: trimmed.to_string(),
            }
        } else {
            DepSource::Registry
        };

        dependencies.push(Dependency {
            name: name.to_string(),
            version: version.to_string(),
            source,
            is_dev: false,
            is_optional: false,
        });
    }

    Ok(DepFile {
        path: path.to_string_lossy().to_string(),
        ecosystem: Ecosystem::Python,
        dependencies,
    })
}

/// Parse a pyproject.toml file.
fn parse_pyproject_toml(path: &Path) -> Result<DepFile> {
    let content =
        fs::read_to_string(path).with_context(|| format!("Failed to read {}", path.display()))?;
    let doc: toml::Value = content
        .parse()
        .with_context(|| format!("Failed to parse TOML from {}", path.display()))?;

    let mut dependencies = Vec::new();

    // Parse [project.dependencies] (PEP 621)
    if let Some(deps) = doc
        .get("project")
        .and_then(|p| p.get("dependencies"))
        .and_then(|d| d.as_array())
    {
        for dep_val in deps {
            if let Some(dep_str) = dep_val.as_str() {
                let (name, version) = parse_python_dep_string(dep_str);
                dependencies.push(Dependency {
                    name,
                    version,
                    source: DepSource::Registry,
                    is_dev: false,
                    is_optional: false,
                });
            }
        }
    }

    // Parse [project.optional-dependencies]
    if let Some(opt_deps) = doc
        .get("project")
        .and_then(|p| p.get("optional-dependencies"))
        .and_then(|d| d.as_table())
    {
        for (_group, deps) in opt_deps {
            if let Some(deps_arr) = deps.as_array() {
                for dep_val in deps_arr {
                    if let Some(dep_str) = dep_val.as_str() {
                        let (name, version) = parse_python_dep_string(dep_str);
                        dependencies.push(Dependency {
                            name,
                            version,
                            source: DepSource::Registry,
                            is_dev: false,
                            is_optional: true,
                        });
                    }
                }
            }
        }
    }

    // Parse [tool.poetry.dependencies] (Poetry format)
    if let Some(deps) = doc
        .get("tool")
        .and_then(|t| t.get("poetry"))
        .and_then(|p| p.get("dependencies"))
        .and_then(|d| d.as_table())
    {
        for (name, value) in deps {
            if name == "python" {
                continue;
            }
            let version = match value {
                toml::Value::String(v) => v.clone(),
                toml::Value::Table(t) => t
                    .get("version")
                    .and_then(|v| v.as_str())
                    .unwrap_or("*")
                    .to_string(),
                _ => "*".to_string(),
            };
            dependencies.push(Dependency {
                name: name.clone(),
                version,
                source: DepSource::Registry,
                is_dev: false,
                is_optional: false,
            });
        }
    }

    Ok(DepFile {
        path: path.to_string_lossy().to_string(),
        ecosystem: Ecosystem::Python,
        dependencies,
    })
}

fn parse_python_dep_string(dep_str: &str) -> (String, String) {
    if let Some(pos) = dep_str.find(|c: char| ['>', '<', '=', '!', '~'].contains(&c)) {
        let name_part = &dep_str[..pos];
        let name = if let Some(bracket_pos) = name_part.find('[') {
            &name_part[..bracket_pos]
        } else {
            name_part
        };
        (name.trim().to_string(), dep_str[pos..].to_string())
    } else {
        let name = if let Some(bracket_pos) = dep_str.find('[') {
            &dep_str[..bracket_pos]
        } else {
            dep_str
        };
        (name.trim().to_string(), "*".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_parse_cargo_toml_basic() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("Cargo.toml");
        fs::write(
            &path,
            r#"
[dependencies]
serde = "1.0"
tokio = { version = "1.0", features = ["full"] }
my-lib = { git = "https://github.com/foo/bar" }
local-crate = { path = "../local" }

[dev-dependencies]
tempfile = "3.0"
"#,
        )
        .unwrap();

        let dep_file = parse_cargo_toml(&path).unwrap();
        assert_eq!(dep_file.ecosystem, Ecosystem::Rust);
        assert_eq!(dep_file.dependencies.len(), 5);

        let serde_dep = dep_file
            .dependencies
            .iter()
            .find(|d| d.name == "serde")
            .unwrap();
        assert_eq!(serde_dep.version, "1.0");
        assert_eq!(serde_dep.source, DepSource::Registry);
        assert!(!serde_dep.is_dev);

        let tokio_dep = dep_file
            .dependencies
            .iter()
            .find(|d| d.name == "tokio")
            .unwrap();
        assert_eq!(tokio_dep.version, "1.0");

        let git_dep = dep_file
            .dependencies
            .iter()
            .find(|d| d.name == "my-lib")
            .unwrap();
        assert!(matches!(git_dep.source, DepSource::Git { .. }));

        let local_dep = dep_file
            .dependencies
            .iter()
            .find(|d| d.name == "local-crate")
            .unwrap();
        assert!(matches!(local_dep.source, DepSource::Path { .. }));

        let tempfile_dep = dep_file
            .dependencies
            .iter()
            .find(|d| d.name == "tempfile")
            .unwrap();
        assert!(tempfile_dep.is_dev);
    }

    #[test]
    fn test_parse_package_json_basic() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("package.json");
        fs::write(
            &path,
            r#"{
    "name": "test-project",
    "dependencies": {
        "express": "^4.18.0",
        "lodash": "~4.17.0"
    },
    "devDependencies": {
        "jest": "^29.0.0"
    },
    "optionalDependencies": {
        "fsevents": "^2.3.0"
    }
}"#,
        )
        .unwrap();

        let dep_file = parse_package_json(&path).unwrap();
        assert_eq!(dep_file.ecosystem, Ecosystem::Node);
        assert_eq!(dep_file.dependencies.len(), 4);

        let express = dep_file
            .dependencies
            .iter()
            .find(|d| d.name == "express")
            .unwrap();
        assert_eq!(express.version, "^4.18.0");
        assert!(!express.is_dev);

        let jest = dep_file
            .dependencies
            .iter()
            .find(|d| d.name == "jest")
            .unwrap();
        assert!(jest.is_dev);

        let fsevents = dep_file
            .dependencies
            .iter()
            .find(|d| d.name == "fsevents")
            .unwrap();
        assert!(fsevents.is_optional);
    }

    #[test]
    fn test_parse_go_mod_basic() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("go.mod");
        fs::write(
            &path,
            r#"module example.com/myapp

go 1.21

require (
    github.com/gin-gonic/gin v1.9.1
    github.com/go-sql-driver/mysql v1.7.0 // indirect
)
"#,
        )
        .unwrap();

        let dep_file = parse_go_mod(&path).unwrap();
        assert_eq!(dep_file.ecosystem, Ecosystem::Go);
        assert_eq!(dep_file.dependencies.len(), 2);

        let gin = dep_file
            .dependencies
            .iter()
            .find(|d| d.name == "github.com/gin-gonic/gin")
            .unwrap();
        assert_eq!(gin.version, "v1.9.1");
        assert!(!gin.is_optional);

        let mysql = dep_file
            .dependencies
            .iter()
            .find(|d| d.name == "github.com/go-sql-driver/mysql")
            .unwrap();
        assert!(mysql.is_optional);
    }

    #[test]
    fn test_parse_requirements_txt_basic() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("requirements.txt");
        fs::write(
            &path,
            r#"# This is a comment
flask>=2.0
requests==2.28.1
django~=4.0
numpy
git+https://github.com/foo/bar.git
"#,
        )
        .unwrap();

        let dep_file = parse_requirements_txt(&path).unwrap();
        assert_eq!(dep_file.ecosystem, Ecosystem::Python);
        assert_eq!(dep_file.dependencies.len(), 5);

        let flask = dep_file
            .dependencies
            .iter()
            .find(|d| d.name == "flask")
            .unwrap();
        assert_eq!(flask.version, ">=2.0");

        let numpy = dep_file
            .dependencies
            .iter()
            .find(|d| d.name == "numpy")
            .unwrap();
        assert_eq!(numpy.version, "*");

        let git_dep = dep_file
            .dependencies
            .iter()
            .find(|d| d.name.starts_with("git+"))
            .unwrap();
        assert!(matches!(git_dep.source, DepSource::Git { .. }));
    }

    #[test]
    fn test_detect_and_parse_multiple_files() {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("Cargo.toml"),
            "[dependencies]\nserde = \"1.0\"\n",
        )
        .unwrap();
        fs::write(
            dir.path().join("package.json"),
            r#"{"dependencies": {"express": "^4.0"}}"#,
        )
        .unwrap();

        let results = detect_and_parse(dir.path()).unwrap();
        assert_eq!(results.len(), 2);
        assert!(results.iter().any(|f| f.ecosystem == Ecosystem::Rust));
        assert!(results.iter().any(|f| f.ecosystem == Ecosystem::Node));
    }

    #[test]
    fn test_parse_pyproject_toml_poetry() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("pyproject.toml");
        fs::write(
            &path,
            r#"
[tool.poetry.dependencies]
python = "^3.9"
flask = "^2.0"
requests = ">=2.28"
"#,
        )
        .unwrap();

        let dep_file = parse_pyproject_toml(&path).unwrap();
        assert_eq!(dep_file.ecosystem, Ecosystem::Python);
        // python should be filtered out
        assert_eq!(dep_file.dependencies.len(), 2);
        assert!(dep_file.dependencies.iter().all(|d| d.name != "python"));
    }
}
