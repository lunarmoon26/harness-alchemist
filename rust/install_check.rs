use std::{
    collections::HashSet,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

use serde::Serialize;
use serde_json::Value;

use crate::{path_utils, process};

const TIMEOUT: Duration = Duration::from_secs(300);
const HARNESSES: &[&str] = &["claude", "codex", "agy", "opencode", "dsh"];

pub fn usage() -> &'static str {
    concat!(
        "Usage: harness-alchemist install-check [project-directory] [options]\n\n",
        "Install-level verification: drives the local harness CLIs (claude, codex,\n",
        "agy, opencode, dsh) against the project's plugin package and asserts each\n",
        "harness can discover it. Static validation is a prerequisite; run\n",
        "'harness-alchemist validate' first.\n\n",
        "Options:\n",
        "  --harness <id>  Limit to one harness (claude, codex, agy, opencode, dsh).\n",
        "                  Repeatable.\n",
        "  --keep          Keep installed plugins and marketplaces after the check.\n",
        "  --json          Print a machine-readable result.\n",
        "  --help          Show this help."
    )
}

struct Args {
    project: Option<String>,
    harnesses: Vec<String>,
    keep: bool,
    json: bool,
    help: bool,
}

fn parse_args(args: &[String]) -> Result<Args, String> {
    let mut parsed = Args {
        project: None,
        harnesses: Vec::new(),
        keep: false,
        json: false,
        help: false,
    };
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--help" | "-h" => {
                parsed.help = true;
                return Ok(parsed);
            }
            "--keep" => parsed.keep = true,
            "--json" => parsed.json = true,
            "--harness" => {
                let value = args
                    .get(index + 1)
                    .filter(|value| !value.starts_with("--"))
                    .ok_or_else(|| "--harness requires a value".to_string())?;
                if !HARNESSES.contains(&value.as_str()) {
                    return Err(format!(
                        "Unknown harness '{value}'. Choose from claude, codex, agy, opencode, dsh"
                    ));
                }
                parsed.harnesses.push(value.clone());
                index += 1;
            }
            value if value.starts_with('-') => return Err(format!("Unknown option: {value}")),
            value if parsed.project.is_some() => {
                let _ = value;
                return Err("Only one project directory may be supplied".to_string());
            }
            value => parsed.project = Some(value.to_string()),
        }
        index += 1;
    }
    Ok(parsed)
}

fn read_json(path: &Path, problems: &mut Vec<String>) -> Option<Value> {
    match fs::read_to_string(path)
        .map_err(|error| error.to_string())
        .and_then(|content| serde_json::from_str(&content).map_err(|error| error.to_string()))
    {
        Ok(value) => Some(value),
        Err(error) => {
            problems.push(format!("{}: invalid JSON ({error})", path.display()));
            None
        }
    }
}

fn skill_names(directory: &Path) -> Vec<String> {
    let Ok(entries) = fs::read_dir(directory) else {
        return Vec::new();
    };
    let mut names = entries
        .flatten()
        .filter_map(|entry| {
            let file_type = entry.file_type().ok()?;
            if !file_type.is_dir() || !entry.path().join("SKILL.md").is_file() {
                return None;
            }
            entry.file_name().into_string().ok()
        })
        .collect::<Vec<_>>();
    names.sort();
    names
}

fn copy_directory(source: &Path, destination: &Path) -> Result<(), String> {
    let root = fs::canonicalize(source).map_err(|error| error.to_string())?;
    copy_directory_inner(source, destination, &root, &mut HashSet::new())
}

fn copy_directory_inner(
    source: &Path,
    destination: &Path,
    root: &Path,
    active_directories: &mut HashSet<PathBuf>,
) -> Result<(), String> {
    let canonical_source = fs::canonicalize(source).map_err(|error| error.to_string())?;
    if !path_utils::is_contained(root, &canonical_source) {
        return Err(format!(
            "Refusing to copy path outside skills root: {}",
            source.display()
        ));
    }
    if !active_directories.insert(canonical_source.clone()) {
        return Err(format!(
            "Refusing to copy symlink cycle: {}",
            source.display()
        ));
    }
    fs::create_dir_all(destination).map_err(|error| error.to_string())?;
    for entry in fs::read_dir(source).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let target = destination.join(entry.file_name());
        let file_type = entry.file_type().map_err(|error| error.to_string())?;
        if file_type.is_dir() {
            copy_directory_inner(&entry.path(), &target, root, active_directories)?;
        } else if file_type.is_file() {
            fs::copy(entry.path(), target).map_err(|error| error.to_string())?;
        } else if file_type.is_symlink() {
            let canonical_target = fs::canonicalize(entry.path())
                .map_err(|error| format!("Invalid symlink {}: {error}", entry.path().display()))?;
            if !path_utils::is_contained(root, &canonical_target) {
                return Err(format!(
                    "Refusing to copy symlink outside skills root: {}",
                    entry.path().display()
                ));
            }
            let target_type = fs::metadata(&canonical_target).map_err(|error| error.to_string())?;
            if target_type.is_dir() {
                copy_directory_inner(&canonical_target, &target, root, active_directories)?;
            } else if target_type.is_file() {
                fs::copy(canonical_target, target).map_err(|error| error.to_string())?;
            } else {
                return Err(format!(
                    "Unsupported symlink target: {}",
                    entry.path().display()
                ));
            }
        } else {
            return Err(format!(
                "Unsupported skill path: {}",
                entry.path().display()
            ));
        }
    }
    active_directories.remove(&canonical_source);
    Ok(())
}

fn execute(program: &str, args: &[String], env: Vec<(OsString, OsString)>) -> process::RunResult {
    process::run(
        program,
        args,
        process::RunOptions {
            env,
            timeout: Some(TIMEOUT),
            ..process::RunOptions::default()
        },
    )
}

fn first_line(text: &str) -> &str {
    text.trim().lines().next().unwrap_or("")
}

fn failure(result: &process::RunResult) -> &str {
    if result.timed_out {
        return "command timed out";
    }
    if result.stderr.trim().is_empty() {
        first_line(&result.stdout)
    } else {
        first_line(&result.stderr)
    }
}

#[derive(Serialize)]
struct HarnessResult {
    harness: String,
    status: String,
    details: Vec<String>,
}

#[derive(Serialize)]
struct Summary {
    pass: usize,
    fail: usize,
    skip: usize,
}

#[derive(Serialize)]
struct InstallOutput<'a> {
    project: &'a Path,
    plugin: &'a str,
    runtime: &'a str,
    results: &'a [HarnessResult],
    summary: Summary,
}

fn record(
    results: &mut Vec<HarnessResult>,
    json: bool,
    harness: &str,
    status: &str,
    details: Vec<String>,
) {
    if !json {
        let marker = match status {
            "pass" => "\u{2714}",
            "skip" => "\u{25cb}",
            _ => "\u{2716}",
        };
        println!("{marker} {harness}: {status}");
        for detail in &details {
            println!("    {detail}");
        }
    }
    results.push(HarnessResult {
        harness: harness.to_string(),
        status: status.to_string(),
        details,
    });
}

pub fn run(args: &[String]) -> i32 {
    let args = match parse_args(args) {
        Ok(args) => args,
        Err(error) => {
            eprintln!("{error}");
            eprintln!("{}", usage());
            return 2;
        }
    };
    if args.help {
        println!("{}", usage());
        return 0;
    }

    let root = if let Some(project) = args.project.as_deref() {
        path_utils::absolute_normalized(Path::new(project)).ok()
    } else {
        std::env::current_dir()
            .ok()
            .and_then(|cwd| path_utils::find_project_root(&cwd))
            .or_else(path_utils::package_root)
    };
    let Some(root) = root else {
        eprintln!("Could not find a universal plugin project. Pass its directory explicitly.");
        return 2;
    };

    let mut problems = Vec::new();
    let layout_path = root.join("alchemy.json");
    let layout = if layout_path.exists() {
        read_json(&layout_path, &mut problems).unwrap_or_else(|| serde_json::json!({}))
    } else {
        serde_json::json!({})
    };
    let plugin_relative = layout
        .get("pluginRoot")
        .and_then(Value::as_str)
        .unwrap_or(".");
    let plugin_root = match path_utils::absolute_normalized(&root.join(plugin_relative)) {
        Ok(path) if path_utils::is_contained(&root, &path) => path,
        _ => {
            eprintln!("alchemy.json pluginRoot escapes the project root");
            return 1;
        }
    };
    if plugin_root.exists()
        && let (Ok(canonical_root), Ok(canonical_plugin)) =
            (fs::canonicalize(&root), fs::canonicalize(&plugin_root))
        && !path_utils::is_contained(&canonical_root, &canonical_plugin)
    {
        eprintln!("alchemy.json pluginRoot resolves through a symlink outside the project root");
        return 1;
    }
    let runtime = layout
        .get("runtime")
        .and_then(Value::as_str)
        .unwrap_or("npm");
    let claude_plugin = read_json(
        &plugin_root.join(".claude-plugin/plugin.json"),
        &mut problems,
    );
    let claude_marketplace =
        read_json(&root.join(".claude-plugin/marketplace.json"), &mut problems);
    if !problems.is_empty() {
        for problem in problems {
            eprintln!("{problem}");
        }
        return 1;
    }
    let plugin_name = claude_plugin
        .as_ref()
        .and_then(|value| value.get("name"))
        .and_then(Value::as_str)
        .unwrap_or("");
    let marketplace_name = claude_marketplace
        .as_ref()
        .and_then(|value| value.get("name"))
        .and_then(Value::as_str)
        .unwrap_or("");
    let skills = skill_names(&plugin_root.join("skills"));
    let selected = if args.harnesses.is_empty() {
        HARNESSES
            .iter()
            .map(|value| value.to_string())
            .collect::<Vec<_>>()
    } else {
        args.harnesses.clone()
    };
    let mut results = Vec::new();

    for harness in selected {
        match harness.as_str() {
            "claude" => {
                let mut details = Vec::new();
                let mut status = "pass";
                let plugin_ref = format!("{plugin_name}@{marketplace_name}");
                let steps = [
                    vec![
                        "plugin".to_string(),
                        "validate".to_string(),
                        plugin_root.to_string_lossy().into_owned(),
                        "--strict".to_string(),
                    ],
                    vec![
                        "plugin".to_string(),
                        "marketplace".to_string(),
                        "add".to_string(),
                        root.to_string_lossy().into_owned(),
                    ],
                    vec![
                        "plugin".to_string(),
                        "install".to_string(),
                        plugin_ref.clone(),
                    ],
                    vec![
                        "plugin".to_string(),
                        "details".to_string(),
                        plugin_name.to_string(),
                    ],
                ];
                let mut marketplace_added = false;
                let mut installed = false;
                for (index, step) in steps.iter().enumerate() {
                    let result = execute("claude", step, Vec::new());
                    if result.missing {
                        status = "skip";
                        details.push("claude CLI not found".to_string());
                        break;
                    }
                    if !result.ok {
                        status = "fail";
                        details.push(format!(
                            "`claude {}` failed: {}",
                            step.join(" "),
                            failure(&result)
                        ));
                        break;
                    }
                    marketplace_added |= index == 1;
                    installed |= index == 2;
                    if index == 3 {
                        for skill in &skills {
                            if !result.stdout.contains(skill) {
                                status = "fail";
                                details
                                    .push(format!("skill '{skill}' missing from plugin details"));
                            }
                        }
                        details.push(format!("{} skills registered", skills.len()));
                    }
                }
                if !args.keep {
                    if installed {
                        execute(
                            "claude",
                            &["plugin".into(), "uninstall".into(), plugin_ref],
                            Vec::new(),
                        );
                    }
                    if marketplace_added {
                        execute(
                            "claude",
                            &[
                                "plugin".into(),
                                "marketplace".into(),
                                "remove".into(),
                                marketplace_name.into(),
                            ],
                            Vec::new(),
                        );
                    }
                }
                record(&mut results, args.json, "claude", status, details);
            }
            "codex" => {
                let mut details = Vec::new();
                let plugin_ref = format!("{plugin_name}@{marketplace_name}");
                let add = execute(
                    "codex",
                    &[
                        "plugin".into(),
                        "marketplace".into(),
                        "add".into(),
                        root.to_string_lossy().into_owned(),
                    ],
                    Vec::new(),
                );
                if add.missing {
                    record(
                        &mut results,
                        args.json,
                        "codex",
                        "skip",
                        vec!["codex CLI not found".to_string()],
                    );
                    continue;
                }
                if !add.ok {
                    record(
                        &mut results,
                        args.json,
                        "codex",
                        "fail",
                        vec![format!("marketplace add failed: {}", failure(&add))],
                    );
                    continue;
                }
                let mut status = "pass";
                let install = execute(
                    "codex",
                    &["plugin".into(), "add".into(), plugin_ref.clone()],
                    Vec::new(),
                );
                let mut installed = false;
                if !install.ok {
                    status = "fail";
                    details.push(format!("plugin add failed: {}", failure(&install)));
                } else {
                    installed = true;
                    let list = execute("codex", &["plugin".into(), "list".into()], Vec::new());
                    if !list.ok || !list.stdout.contains(&plugin_ref) {
                        status = "fail";
                        details.push("plugin not listed as installed".to_string());
                    } else if !list.stdout.contains("installed, enabled") {
                        status = "fail";
                        details.push("plugin installed but not enabled".to_string());
                    } else {
                        details.push("installed and enabled".to_string());
                    }
                }
                if !args.keep {
                    if installed {
                        execute(
                            "codex",
                            &["plugin".into(), "remove".into(), plugin_ref],
                            Vec::new(),
                        );
                    }
                    execute(
                        "codex",
                        &[
                            "plugin".into(),
                            "marketplace".into(),
                            "remove".into(),
                            marketplace_name.into(),
                        ],
                        Vec::new(),
                    );
                }
                record(&mut results, args.json, "codex", status, details);
            }
            "agy" => {
                let validate = execute(
                    "agy",
                    &[
                        "plugin".into(),
                        "validate".into(),
                        plugin_root.to_string_lossy().into_owned(),
                    ],
                    Vec::new(),
                );
                if validate.missing {
                    record(
                        &mut results,
                        args.json,
                        "agy",
                        "skip",
                        vec!["agy CLI not found".to_string()],
                    );
                    continue;
                }
                if !validate.ok {
                    record(
                        &mut results,
                        args.json,
                        "agy",
                        "fail",
                        vec![format!("validate failed: {}", failure(&validate))],
                    );
                    continue;
                }
                let install = execute(
                    "agy",
                    &[
                        "plugin".into(),
                        "install".into(),
                        plugin_root.to_string_lossy().into_owned(),
                    ],
                    Vec::new(),
                );
                if !install.ok {
                    record(
                        &mut results,
                        args.json,
                        "agy",
                        "fail",
                        vec![format!("install failed: {}", failure(&install))],
                    );
                    continue;
                }
                let list = execute("agy", &["plugin".into(), "list".into()], Vec::new());
                let (status, details) = if !list.ok || !list.stdout.contains(plugin_name) {
                    (
                        "fail",
                        vec!["plugin missing from agy plugin list".to_string()],
                    )
                } else {
                    ("pass", vec!["validated, installed, and listed".to_string()])
                };
                if !args.keep {
                    execute(
                        "agy",
                        &["plugin".into(), "uninstall".into(), plugin_name.into()],
                        Vec::new(),
                    );
                }
                record(&mut results, args.json, "agy", status, details);
            }
            "opencode" => {
                let adapter = plugin_root.join("dist/opencode.js");
                if runtime == "npm" && !adapter.exists() {
                    record(
                        &mut results,
                        args.json,
                        "opencode",
                        "fail",
                        vec![
                            format!(
                                "npm runtime requires a built adapter; missing {}",
                                adapter.display()
                            ),
                            "run `npm run build` in the plugin package first".to_string(),
                        ],
                    );
                    continue;
                }
                let isolation = match tempfile::Builder::new().prefix("ha-opencode-").tempdir() {
                    Ok(directory) => directory,
                    Err(error) => {
                        record(
                            &mut results,
                            args.json,
                            "opencode",
                            "fail",
                            vec![error.to_string()],
                        );
                        continue;
                    }
                };
                let config = isolation.path().join("opencode");
                let skills_directory = config.join("skills");
                if let Err(error) = copy_directory(&plugin_root.join("skills"), &skills_directory) {
                    record(&mut results, args.json, "opencode", "fail", vec![error]);
                    continue;
                }
                let mut config_json =
                    serde_json::json!({ "$schema": "https://opencode.ai/config.json" });
                if runtime == "npm" {
                    config_json["plugin"] =
                        serde_json::json!([format!("file://{}", adapter.display())]);
                }
                if let Err(error) = fs::write(
                    config.join("opencode.json"),
                    format!("{}\n", serde_json::to_string_pretty(&config_json).unwrap()),
                ) {
                    record(
                        &mut results,
                        args.json,
                        "opencode",
                        "fail",
                        vec![error.to_string()],
                    );
                    continue;
                }
                let env = vec![
                    (
                        OsString::from("XDG_CONFIG_HOME"),
                        isolation.path().as_os_str().to_os_string(),
                    ),
                    (
                        OsString::from("HOME"),
                        isolation.path().as_os_str().to_os_string(),
                    ),
                ];
                let skill_check =
                    execute("opencode", &["debug".into(), "skill".into()], env.clone());
                if skill_check.missing {
                    record(
                        &mut results,
                        args.json,
                        "opencode",
                        "skip",
                        vec!["opencode CLI not found".to_string()],
                    );
                    continue;
                }
                let mut details = Vec::new();
                let mut status = "pass";
                let missing = skills
                    .iter()
                    .filter(|skill| {
                        !skill_check
                            .stdout
                            .contains(&format!("\"name\": \"{skill}\""))
                    })
                    .cloned()
                    .collect::<Vec<_>>();
                if missing.is_empty() {
                    details.push(format!("{} skills discovered", skills.len()));
                } else {
                    status = "fail";
                    details.push(format!("skills not discovered: {}", missing.join(", ")));
                }
                if runtime == "npm" && status == "pass" {
                    let startup = execute("opencode", &["debug".into(), "startup".into()], env);
                    if startup.ok {
                        details.push("plugin loaded at startup".to_string());
                    } else {
                        status = "fail";
                        details.push(format!("startup failed: {}", failure(&startup)));
                    }
                }
                record(&mut results, args.json, "opencode", status, details);
            }
            "dsh" => {
                if runtime != "npm" {
                    record(&mut results, args.json, "dsh", "skip", vec!["skills runtime has no Cordis bundle; install skills via filesystem roots instead".to_string()]);
                    continue;
                }
                let adapter = plugin_root.join("dist/deepseek.js");
                if !adapter.exists() {
                    record(
                        &mut results,
                        args.json,
                        "dsh",
                        "fail",
                        vec![
                            format!(
                                "npm runtime requires built adapters; missing {}",
                                adapter.display()
                            ),
                            "run `npm run build` in the plugin package first".to_string(),
                        ],
                    );
                    continue;
                }
                let isolation = match tempfile::Builder::new().prefix("ha-dsh-").tempdir() {
                    Ok(directory) => directory,
                    Err(error) => {
                        record(
                            &mut results,
                            args.json,
                            "dsh",
                            "fail",
                            vec![error.to_string()],
                        );
                        continue;
                    }
                };
                let env = vec![(
                    OsString::from("DSH_HOME"),
                    isolation.path().as_os_str().to_os_string(),
                )];
                let direct = execute("dsh", &["--version".into()], env.clone());
                let (command, prefix) = if direct.missing {
                    (
                        "npx",
                        vec!["-y".to_string(), "@deepseek-ai/dsh".to_string()],
                    )
                } else {
                    ("dsh", Vec::new())
                };
                let mut add_args = prefix.clone();
                add_args.extend([
                    "plugin".into(),
                    "--profile".into(),
                    "install-check".into(),
                    "add".into(),
                    plugin_root.to_string_lossy().into_owned(),
                ]);
                let add = execute(command, &add_args, env.clone());
                if add.missing {
                    record(
                        &mut results,
                        args.json,
                        "dsh",
                        "skip",
                        vec!["dsh CLI not found and npx unavailable".to_string()],
                    );
                    continue;
                }
                if !add.ok {
                    record(
                        &mut results,
                        args.json,
                        "dsh",
                        "fail",
                        vec![format!("profile add failed: {}", failure(&add))],
                    );
                    continue;
                }
                let mut dump_args = prefix;
                dump_args.extend([
                    "--profile".into(),
                    "install-check".into(),
                    "--dump-config".into(),
                ]);
                let dump = execute(command, &dump_args, env);
                let (status, details) =
                    if dump.ok && dump.stdout.contains(&format!("- id: {plugin_name}")) {
                        (
                            "pass",
                            vec!["cordis bundle composed into isolated profile".to_string()],
                        )
                    } else {
                        (
                            "fail",
                            vec!["cordis insert entry missing from composed profile".to_string()],
                        )
                    };
                record(&mut results, args.json, "dsh", status, details);
            }
            _ => unreachable!(),
        }
    }

    let failed = results
        .iter()
        .filter(|result| result.status == "fail")
        .count();
    let skipped = results
        .iter()
        .filter(|result| result.status == "skip")
        .count();
    if args.json {
        let output = InstallOutput {
            project: &root,
            plugin: plugin_name,
            runtime,
            results: &results,
            summary: Summary {
                pass: results.len() - failed - skipped,
                fail: failed,
                skip: skipped,
            },
        };
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
    } else if failed == 0 {
        println!(
            "Install check passed for {plugin_name} ({} verified, {skipped} skipped)",
            results.len() - skipped
        );
    } else {
        println!(
            "Install check failed for {plugin_name}: {failed} of {} harnesses failed",
            results.len()
        );
    }
    i32::from(failed > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unknown_harness() {
        assert!(parse_args(&["--harness".into(), "cursor".into()]).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn copies_in_tree_symlinks_and_rejects_escapes() {
        use std::os::unix::fs::symlink;

        let temporary = tempfile::tempdir().expect("temporary directory");
        let source = temporary.path().join("skills");
        let destination = temporary.path().join("copied");
        fs::create_dir_all(source.join("shared")).expect("source directories");
        fs::write(source.join("shared/main.mjs"), "export {}\n").expect("source file");
        symlink("shared/main.mjs", source.join("main.mjs")).expect("in-tree symlink");

        copy_directory(&source, &destination).expect("copy in-tree symlink");
        assert_eq!(
            fs::read_to_string(destination.join("main.mjs")).expect("copied link target"),
            "export {}\n"
        );

        let outside = temporary.path().join("outside.mjs");
        fs::write(&outside, "outside\n").expect("outside file");
        symlink(&outside, source.join("outside.mjs")).expect("escaping symlink");
        assert!(
            copy_directory(&source, &temporary.path().join("rejected"))
                .expect_err("reject escaping symlink")
                .contains("outside skills root")
        );
    }
}
