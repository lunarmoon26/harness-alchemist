use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use chrono::{Datelike, SecondsFormat, Utc};
use regex::Regex;
use serde::Serialize;
use url::Url;
use uuid::Uuid;

use crate::{assets, path_utils, validate};

const CANONICAL_VERSION: &str = "v0.1.0";

pub fn list_templates() -> &'static str {
    "v0.1.0 (canonical)"
}

pub fn usage() -> &'static str {
    concat!(
        "Usage: harness-alchemist create <output-directory> [options]\n\n",
        "Create one TypeScript repository for Claude Code, Codex/ChatGPT, OpenCode,\n",
        "Google Antigravity, and DeepSeek Harness/Cordis.\n\n",
        "Required:\n",
        "  --description <text>       One-sentence plugin description.\n",
        "  --author <name>            Author or team name.\n",
        "  --repository <source>      GitHub owner/repo or a full Git URL.\n\n",
        "Options:\n",
        "  --name <name>              Plugin name; defaults to output directory name.\n",
        "  --package <name>           npm package; defaults to plugin name. Its basename\n",
        "                             must equal the plugin name.\n",
        "  --display-name <text>      Human-readable name; defaults from plugin name.\n",
        "  --marketplace <name>       Claude/Codex marketplace name; defaults to\n",
        "                              <plugin-name>-plugins.\n",
        "  --template <version>       Canonical template version (default: v0.1.0).\n",
        "  --license <id>             MIT (default), Apache-2.0, or UNLICENSED.\n",
        "  --dry-run                  Validate inputs and print the files without writing.\n",
        "  --help                     Show this help.\n\n",
        "The destination must be missing or empty. Existing content is never replaced."
    )
}

#[derive(Serialize)]
struct DryRunOutput<'a> {
    output: &'a Path,
    template: &'a str,
    plugin: &'a str,
    package: &'a str,
    files: Vec<String>,
}

#[derive(Default)]
struct RawOptions {
    output: Option<String>,
    name: Option<String>,
    description: Option<String>,
    package_name: Option<String>,
    author: Option<String>,
    repository: Option<String>,
    display_name: Option<String>,
    marketplace: Option<String>,
    template_version: String,
    license: String,
    dry_run: bool,
    help: bool,
}

struct Repository {
    install_source: String,
    url: String,
}

struct Options {
    output: PathBuf,
    name: String,
    description: String,
    package_name: String,
    author: String,
    repository: Repository,
    display_name: String,
    marketplace: String,
    template_version: String,
    license: String,
    dry_run: bool,
}

fn parse_args(args: &[String]) -> Result<RawOptions, String> {
    let mut options = RawOptions {
        template_version: CANONICAL_VERSION.to_string(),
        license: "MIT".to_string(),
        ..RawOptions::default()
    };
    let mut index = 0;
    while index < args.len() {
        let argument = &args[index];
        if argument == "--help" || argument == "-h" {
            options.help = true;
            return Ok(options);
        }
        if argument == "--dry-run" {
            options.dry_run = true;
            index += 1;
            continue;
        }
        if matches!(
            argument.as_str(),
            "--name"
                | "--description"
                | "--package"
                | "--author"
                | "--repository"
                | "--display-name"
                | "--marketplace"
                | "--template"
                | "--license"
        ) {
            let value = args.get(index + 1).filter(|value| !value.starts_with("--"));
            let Some(value) = value else {
                return Err(format!("{argument} requires a value"));
            };
            match argument.as_str() {
                "--name" => options.name = Some(value.clone()),
                "--description" => options.description = Some(value.clone()),
                "--package" => options.package_name = Some(value.clone()),
                "--author" => options.author = Some(value.clone()),
                "--repository" => options.repository = Some(value.clone()),
                "--display-name" => options.display_name = Some(value.clone()),
                "--marketplace" => options.marketplace = Some(value.clone()),
                "--template" => options.template_version = value.clone(),
                "--license" => options.license = value.clone(),
                _ => unreachable!(),
            }
            index += 2;
            continue;
        }
        if argument.starts_with('-') {
            return Err(format!("Unknown option: {argument}"));
        }
        if options.output.is_some() {
            return Err("Only one output directory may be supplied".to_string());
        }
        options.output = Some(argument.clone());
        index += 1;
    }
    Ok(options)
}

fn title_case(name: &str) -> String {
    name.split('-')
        .map(|part| {
            let mut characters = part.chars();
            match characters.next() {
                Some(first) => first.to_uppercase().collect::<String>() + characters.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn normalize_repository(source: &str) -> Result<Repository, String> {
    let shorthand = Regex::new(r"^[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+$").unwrap();
    if shorthand.is_match(source) {
        return Ok(Repository {
            install_source: source.to_string(),
            url: format!("https://github.com/{source}"),
        });
    }
    if let Ok(url) = Url::parse(source) {
        let path = url.path().trim_matches('/').trim_end_matches(".git");
        let pieces = path.split('/').collect::<Vec<_>>();
        let install_source = if url.host_str() == Some("github.com") && pieces.len() == 2 {
            format!("{}/{}", pieces[0], pieces[1])
        } else {
            source.to_string()
        };
        return Ok(Repository {
            install_source,
            url: source.strip_suffix(".git").unwrap_or(source).to_string(),
        });
    }
    if Regex::new(r"^git@[^:]+:.+").unwrap().is_match(source) {
        return Ok(Repository {
            install_source: source.to_string(),
            url: source.to_string(),
        });
    }
    Err("--repository must be GitHub owner/repo or a full Git URL".to_string())
}

fn validate_options(raw: RawOptions) -> Result<Options, String> {
    let output = raw
        .output
        .ok_or_else(|| "An output directory is required".to_string())?;
    let output = path_utils::absolute_normalized(Path::new(&output))?;
    let name = raw.name.unwrap_or_else(|| {
        output
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned()
    });
    let name_pattern = Regex::new(r"^[a-z0-9]+(?:-[a-z0-9]+)*$").unwrap();
    if !name_pattern.is_match(&name) || name.chars().count() > 56 {
        return Err(
            "Plugin name must be <=56 characters using lowercase letters, digits, and single hyphens"
                .to_string(),
        );
    }
    let package_name = raw.package_name.unwrap_or_else(|| name.clone());
    let package_pattern =
        Regex::new(r"^(?:@[a-z0-9][a-z0-9._-]*/)?[a-z0-9]+(?:-[a-z0-9]+)*$").unwrap();
    if !package_pattern.is_match(&package_name)
        || package_name.rsplit('/').next() != Some(name.as_str())
    {
        return Err(
            "npm package must be unscoped or @scoped and have the plugin name as its basename"
                .to_string(),
        );
    }
    let description = raw.description.unwrap_or_default();
    if description.trim().is_empty() {
        return Err("--description is required".to_string());
    }
    if description.contains('\n') {
        return Err("--description must be one line".to_string());
    }
    let author = raw.author.unwrap_or_default();
    if author.trim().is_empty() {
        return Err("--author is required".to_string());
    }
    let repository_source = raw.repository.unwrap_or_default();
    if repository_source.trim().is_empty() {
        return Err("--repository is required".to_string());
    }
    if !matches!(raw.license.as_str(), "MIT" | "Apache-2.0" | "UNLICENSED") {
        return Err("--license must be MIT, Apache-2.0, or UNLICENSED".to_string());
    }
    if raw.template_version != CANONICAL_VERSION {
        return Err(format!(
            "Unknown template version '{}'. Available: {CANONICAL_VERSION}",
            raw.template_version
        ));
    }
    let marketplace = raw.marketplace.unwrap_or_else(|| format!("{name}-plugins"));
    if !name_pattern.is_match(&marketplace) || marketplace.chars().count() > 64 {
        return Err("Marketplace name must be <=64 characters in lowercase kebab-case".to_string());
    }
    let display_name = raw
        .display_name
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| title_case(&name));

    Ok(Options {
        output,
        name,
        description: description.trim().to_string(),
        package_name,
        author: author.trim().to_string(),
        repository: normalize_repository(repository_source.trim())?,
        display_name,
        marketplace,
        template_version: raw.template_version,
        license: raw.license,
        dry_run: raw.dry_run,
    })
}

fn destination_path(source: &str, name: &str) -> String {
    source
        .split('/')
        .map(|segment| match segment {
            "shared-skill" => name.to_string(),
            "develop-template" => format!("develop-{name}"),
            _ => segment.strip_suffix(".tpl").unwrap_or(segment).to_string(),
        })
        .collect::<Vec<_>>()
        .join("/")
}

fn json_string(value: &str) -> String {
    serde_json::to_string(value).expect("serialize template string")
}

fn render(content: &str, tokens: &HashMap<&str, String>) -> Result<String, String> {
    let token_pattern = Regex::new(r"\{\{([A-Z0-9_]+)\}\}").unwrap();
    let mut error = None;
    let rendered = token_pattern.replace_all(content, |captures: &regex::Captures<'_>| {
        let key = &captures[1];
        match tokens.get(key) {
            Some(value) => value.clone(),
            None => {
                error = Some(format!("Unknown template token {}", &captures[0]));
                captures[0].to_string()
            }
        }
    });
    match error {
        Some(error) => Err(error),
        None => Ok(rendered.into_owned()),
    }
}

fn template_tokens(options: &Options) -> HashMap<&'static str, String> {
    let now = Utc::now();
    let tool_name = format!("{}_run", options.name.replace('-', "_"));
    let mut tokens = HashMap::new();
    for (key, value) in [
        ("NAME", options.name.clone()),
        ("NAME_JSON", json_string(&options.name)),
        ("TOOL_NAME", tool_name.clone()),
        ("TOOL_NAME_JSON", json_string(&tool_name)),
        ("DISPLAY_NAME", options.display_name.clone()),
        ("DISPLAY_NAME_JSON", json_string(&options.display_name)),
        ("DESCRIPTION", options.description.clone()),
        ("DESCRIPTION_JSON", json_string(&options.description)),
        ("TEMPLATE_VERSION", options.template_version.clone()),
        (
            "GENERATOR_VERSION",
            env!("HARNESS_ALCHEMIST_VERSION").to_string(),
        ),
        (
            "CREATED_AT",
            now.to_rfc3339_opts(SecondsFormat::Millis, true),
        ),
        (
            "SHARED_SKILL_DESCRIPTION_JSON",
            json_string(&format!(
                "{} Use when the user requests {} workflows or explicitly asks to use the {} plugin.",
                options.description, options.display_name, options.name
            )),
        ),
        (
            "DEVELOPMENT_SKILL_DESCRIPTION_JSON",
            json_string(&format!(
                "Develop, validate, and publish the {} universal coding-agent plugin. Use when modifying its shared skills, Claude or Codex manifests, OpenCode npm entrypoint, Antigravity bundle, or DeepSeek Cordis integration.",
                options.display_name
            )),
        ),
        ("PACKAGE_NAME", options.package_name.clone()),
        ("PACKAGE_NAME_JSON", json_string(&options.package_name)),
        ("AUTHOR", options.author.clone()),
        ("AUTHOR_JSON", json_string(&options.author)),
        (
            "REPOSITORY_SOURCE",
            options.repository.install_source.clone(),
        ),
        (
            "REPOSITORY_SOURCE_JSON",
            json_string(&options.repository.install_source),
        ),
        ("REPOSITORY_URL", options.repository.url.clone()),
        ("REPOSITORY_URL_JSON", json_string(&options.repository.url)),
        ("MARKETPLACE", options.marketplace.clone()),
        ("MARKETPLACE_JSON", json_string(&options.marketplace)),
        ("LICENSE", options.license.clone()),
        ("LICENSE_JSON", json_string(&options.license)),
        ("YEAR", now.year().to_string()),
    ] {
        tokens.insert(key, value);
    }
    tokens
}

fn license_text(options: &Options) -> String {
    if options.license == "UNLICENSED" {
        return format!(
            "Copyright (c) {} {}\n\nAll rights reserved. This project is not licensed for redistribution.\n",
            Utc::now().year(),
            options.author
        );
    }
    if options.license == "Apache-2.0" {
        return assets::APACHE_LICENSE.to_string();
    }
    format!(
        "MIT License\n\nCopyright (c) {} {}\n\nPermission is hereby granted, free of charge, to any person obtaining a copy\n\
of this software and associated documentation files (the \"Software\"), to deal\n\
in the Software without restriction, including without limitation the rights\n\
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell\n\
copies of the Software, and to permit persons to whom the Software is\n\
furnished to do so, subject to the following conditions:\n\n\
The above copyright notice and this permission notice shall be included in all\n\
copies or substantial portions of the Software.\n\n\
THE SOFTWARE IS PROVIDED \"AS IS\", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR\n\
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,\n\
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE\n\
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER\n\
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,\n\
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE\n\
SOFTWARE.\n",
        Utc::now().year(),
        options.author
    )
}

#[cfg(unix)]
fn make_executable(path: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    let mut permissions = fs::metadata(path)
        .map_err(|error| error.to_string())?
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).map_err(|error| error.to_string())
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) -> Result<(), String> {
    Ok(())
}

fn render_tree(destination: &Path, options: &Options) -> Result<(), String> {
    let tokens = template_tokens(options);
    for asset in assets::TEMPLATE_ASSETS {
        let relative = destination_path(asset.path, &options.name);
        let target = destination.join(&relative);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        fs::write(&target, render(asset.content, &tokens)?).map_err(|error| error.to_string())?;
        if relative.starts_with(&format!("skills/{}/scripts/", options.name)) {
            make_executable(&target)?;
        }
    }
    let local_scripts = destination
        .join(".agents/skills")
        .join(format!("develop-{}", options.name))
        .join("scripts");
    fs::create_dir_all(&local_scripts).map_err(|error| error.to_string())?;
    fs::write(
        local_scripts.join("validate.mjs"),
        assets::GENERATED_VALIDATOR,
    )
    .map_err(|error| error.to_string())?;
    fs::write(destination.join("LICENSE"), license_text(options))
        .map_err(|error| error.to_string())?;
    Ok(())
}

fn generated_files(options: &Options) -> Vec<String> {
    let mut files = assets::TEMPLATE_ASSETS
        .iter()
        .map(|asset| destination_path(asset.path, &options.name))
        .collect::<Vec<_>>();
    files.push(format!(
        ".agents/skills/develop-{}/scripts/validate.mjs",
        options.name
    ));
    files.push("LICENSE".to_string());
    files.sort();
    files.dedup();
    files
}

pub fn run(args: &[String]) -> i32 {
    let parsed = match parse_args(args) {
        Ok(parsed) => parsed,
        Err(error) => {
            eprintln!("{error}");
            eprintln!("{}", usage());
            return 2;
        }
    };
    if parsed.help {
        println!("{}", usage());
        return 0;
    }
    let options = match validate_options(parsed) {
        Ok(options) => options,
        Err(error) => {
            eprintln!("{error}");
            eprintln!("{}", usage());
            return 2;
        }
    };

    let state = match fs::metadata(&options.output) {
        Ok(metadata) if !metadata.is_dir() => "not-directory",
        Ok(_) => match fs::read_dir(&options.output) {
            Ok(mut entries) => {
                if entries.next().is_none() {
                    "empty"
                } else {
                    "non-empty"
                }
            }
            Err(error) => {
                eprintln!("{error}");
                return 1;
            }
        },
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => "missing",
        Err(error) => {
            eprintln!("{error}");
            return 1;
        }
    };
    if state == "not-directory" || state == "non-empty" {
        eprintln!(
            "Destination must be missing or empty: {}",
            options.output.display()
        );
        return 1;
    }

    if options.dry_run {
        let output = DryRunOutput {
            output: &options.output,
            template: &options.template_version,
            plugin: &options.name,
            package: &options.package_name,
            files: generated_files(&options),
        };
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
        return 0;
    }

    let Some(parent) = options.output.parent() else {
        eprintln!("Output directory has no parent");
        return 1;
    };
    if let Err(error) = fs::create_dir_all(parent) {
        eprintln!("{error}");
        return 1;
    }
    let base = options
        .output
        .file_name()
        .unwrap_or_default()
        .to_string_lossy();
    let staging = parent.join(format!(".{base}.scaffold-{}", Uuid::new_v4()));
    let result = (|| -> Result<(), String> {
        fs::create_dir(&staging).map_err(|error| error.to_string())?;
        render_tree(&staging, &options)?;
        let validation = validate::validate_project(&staging, false);
        if !validation.errors.is_empty() {
            return Err(format!(
                "Generated project failed validation:\n{}",
                validation.errors.join("\n")
            ));
        }
        if state == "empty" {
            fs::remove_dir(&options.output).map_err(|error| error.to_string())?;
        }
        fs::rename(&staging, &options.output).map_err(|error| error.to_string())?;
        Ok(())
    })();
    if let Err(error) = result {
        let _ = fs::remove_dir_all(&staging);
        eprintln!("{error}");
        return 1;
    }

    println!(
        "Created universal plugin scaffold at {}",
        options.output.display()
    );
    println!(
        "Next: cd {} && npm install && npm run verify",
        json_string(&options.output.to_string_lossy())
    );
    println!(
        "Bun:  cd {} && bun install && bun run verify",
        json_string(&options.output.to_string_lossy())
    );
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_github_repositories() {
        let repository = normalize_repository("example/plugin").unwrap();
        assert_eq!(repository.install_source, "example/plugin");
        assert_eq!(repository.url, "https://github.com/example/plugin");
    }

    #[test]
    fn lists_each_generated_file_once() {
        let options = validate_options(RawOptions {
            output: Some("example-plugin".to_string()),
            description: Some("Example".to_string()),
            author: Some("Example".to_string()),
            repository: Some("example/plugin".to_string()),
            template_version: CANONICAL_VERSION.to_string(),
            license: "MIT".to_string(),
            ..RawOptions::default()
        })
        .unwrap();
        let files = generated_files(&options);
        assert_eq!(
            files.len(),
            files.iter().collect::<std::collections::HashSet<_>>().len()
        );
        assert!(!files.iter().any(|path| path.ends_with(".DS_Store")));
    }
}
