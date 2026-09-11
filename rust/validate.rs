use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

use chrono::{DateTime, NaiveDate, NaiveDateTime};
use regex::Regex;
use ruff_python_parser::parse_module;
use serde::Serialize;
use serde_json::{Map, Value};

use crate::{path_utils, process};

const AGENT_PLUGIN_SCHEMA: &str = "https://agent-plugins.org/schemas/1.0.0/plugin.schema.json";
const AGENT_PLUGIN_MCP_SCHEMA: &str = "https://agent-plugins.org/schemas/1.0.0/mcp.schema.json";
const SKILL_RUNTIME_PACKAGE: &str = "@lunarmoon26/agent-skill-runtime";
const SKILL_RUNTIME_VERSION: &str = "0.1.1";
const RUNTIME_MODES: &[&str] = &["npm", "skills"];

const PROJECT_REQUIRED_FILES: &[&str] = &[
    ".agents/plugins/marketplace.json",
    ".claude-plugin/marketplace.json",
    ".gitignore",
    "AGENTS.md",
    "LICENSE",
    "README.md",
];
const PLUGIN_REQUIRED_FILES: &[&str] = &[
    ".claude-plugin/plugin.json",
    ".codex-plugin/plugin.json",
    "mcp.json",
    "plugin.json",
];
const NPM_PLUGIN_REQUIRED_FILES: &[&str] = &[
    "cordis.patch.yml",
    "package.json",
    "src/deepseek.ts",
    "src/opencode.ts",
    "tsconfig.json",
];

#[derive(Serialize)]
pub struct ValidationResult {
    pub root: String,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Serialize)]
struct ValidationOutput<'a> {
    valid: bool,
    root: &'a str,
    errors: &'a [String],
    warnings: &'a [String],
}

pub fn usage() -> &'static str {
    concat!(
        "Usage: harness-alchemist validate [project-directory] [--external] [--json]\n\n",
        "Validates a universal Claude, Codex, OpenCode, Antigravity, and DeepSeek\n",
        "plugin scaffold. By default the project is resolved from the current working\n",
        "directory or from the installed package.\n\n",
        "Product skills are checked against the Agent Skills specification, and every\n",
        "present runtime manifest is validated. Python entrypoints are syntax-checked\n",
        "without executing project code.\n\n",
        "Options:\n",
        "  --external  Run installed platform validators, currently Claude Code.\n",
        "  --json      Print a machine-readable result.\n",
        "  --help      Show this help."
    )
}

struct Args {
    project: Option<String>,
    external: bool,
    json: bool,
    help: bool,
}

fn parse_args(args: &[String]) -> Result<Args, String> {
    let mut parsed = Args {
        project: None,
        external: false,
        json: false,
        help: false,
    };
    for argument in args {
        match argument.as_str() {
            "--help" | "-h" => {
                parsed.help = true;
                return Ok(parsed);
            }
            "--external" => parsed.external = true,
            "--json" => parsed.json = true,
            value if value.starts_with('-') => return Err(format!("Unknown option: {value}")),
            value if parsed.project.is_some() => {
                let _ = value;
                return Err("Only one project directory may be supplied".to_string());
            }
            value => parsed.project = Some(value.to_string()),
        }
    }
    Ok(parsed)
}

fn read_json(path: &Path, errors: &mut Vec<String>) -> Option<Value> {
    match fs::read_to_string(path) {
        Ok(content) => match serde_json::from_str(&content) {
            Ok(value) => Some(value),
            Err(error) => {
                errors.push(format!("{}: invalid JSON ({error})", path.display()));
                None
            }
        },
        Err(error) => {
            errors.push(format!("{}: invalid JSON ({error})", path.display()));
            None
        }
    }
}

fn require_path(root: &Path, relative: &str, errors: &mut Vec<String>) {
    if !root.join(relative).exists() {
        errors.push(format!("Missing required file: {relative}"));
    }
}

fn object<'a>(
    value: &'a Value,
    label: &str,
    errors: &mut Vec<String>,
) -> Option<&'a Map<String, Value>> {
    match value.as_object() {
        Some(value) => Some(value),
        None => {
            errors.push(format!("{label} must contain a JSON object"));
            None
        }
    }
}

fn string<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

fn export_target<'a>(exports: &'a Value, key: &str, field: &str) -> Option<&'a str> {
    let value = exports.get(key)?;
    value
        .as_str()
        .or_else(|| value.get(field).and_then(Value::as_str))
}

fn array_has(value: &Value, expected: &str) -> bool {
    value
        .as_array()
        .is_some_and(|values| values.iter().any(|value| value.as_str() == Some(expected)))
}

fn require_manifest_path(root: &Path, value: Option<&str>, label: &str, errors: &mut Vec<String>) {
    let Some(value) = value.filter(|value| value.starts_with("./")) else {
        errors.push(format!("{label} must be a ./-relative path"));
        return;
    };
    let target = match path_utils::absolute_normalized(&root.join(value)) {
        Ok(target) => target,
        Err(error) => {
            errors.push(error);
            return;
        }
    };
    if !path_utils::is_contained(root, &target) {
        errors.push(format!("{label} escapes the plugin root"));
        return;
    }
    if !target.exists() {
        errors.push(format!("{label} points to missing path {value}"));
        return;
    }
    let Ok(canonical_root) = fs::canonicalize(root) else {
        return;
    };
    let Ok(canonical_target) = fs::canonicalize(&target) else {
        return;
    };
    if !path_utils::is_contained(&canonical_root, &canonical_target) {
        errors.push(format!(
            "{label} resolves through a symlink outside the plugin root"
        ));
    }
}

fn unquote(value: &str) -> String {
    let value = value.trim();
    if value.len() >= 2
        && ((value.starts_with('"') && value.ends_with('"'))
            || (value.starts_with('\'') && value.ends_with('\'')))
    {
        value[1..value.len() - 1].to_string()
    } else {
        value.to_string()
    }
}

fn parse_frontmatter(content: &str) -> Option<HashMap<String, String>> {
    let content = content.strip_prefix("---\n")?;
    let end = content.find("\n---")?;
    let pattern = Regex::new(r"^([a-zA-Z0-9-]+):\s*(.*)$").unwrap();
    let mut values = HashMap::new();
    for line in content[..end].lines() {
        if let Some(captures) = pattern.captures(line) {
            values.insert(captures[1].to_string(), unquote(&captures[2]));
        }
    }
    Some(values)
}

fn valid_created_at(value: &str) -> bool {
    DateTime::parse_from_rfc3339(value).is_ok()
        || DateTime::parse_from_str(value, "%Y-%m-%dT%H:%M%#z").is_ok()
        || NaiveDate::parse_from_str(value, "%Y-%m-%d").is_ok()
        || NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M").is_ok()
        || NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S%.f").is_ok()
}

fn yaml_scalar(value: &str) -> Option<String> {
    let value = value.trim();
    if value.starts_with('\'') && value.ends_with('\'') && value.len() >= 2 {
        return Some(value[1..value.len() - 1].replace("''", "'"));
    }
    if value.starts_with('"') && value.ends_with('"') {
        return serde_json::from_str::<String>(value).ok();
    }
    Regex::new(r"^[A-Za-z0-9@/_.-]+$")
        .unwrap()
        .is_match(value)
        .then(|| value.to_string())
}

struct CordisEntry {
    id: String,
    name: Option<String>,
    valid: bool,
}

fn leading_spaces(value: &str) -> usize {
    value.len() - value.trim_start_matches(' ').len()
}

fn parse_cordis_entries(content: &str) -> Vec<CordisEntry> {
    if content.contains('\t') {
        return Vec::new();
    }
    let lines = content.lines().collect::<Vec<_>>();
    let insert = Regex::new(r"^-\s+insert:\s*(?:#.*)?$").unwrap();
    let id_pattern = Regex::new(r"^(\s*)-\s+id:\s*(.+?)\s*$").unwrap();
    let name_pattern = Regex::new(r"^(\s*)name:\s*(.+?)\s*$").unwrap();
    let mut entries = Vec::new();

    for (index, line) in lines.iter().enumerate() {
        if !insert.is_match(line) {
            continue;
        }
        let mut entry_indent = None;
        let mut next = index + 1;
        while next < lines.len() {
            let line = lines[next];
            if !line.trim().is_empty() && leading_spaces(line) == 0 {
                break;
            }
            if line.trim().is_empty() || line.trim_start().starts_with('#') {
                next += 1;
                continue;
            }
            let Some(captures) = id_pattern.captures(line) else {
                if entry_indent.is_none() {
                    break;
                }
                next += 1;
                continue;
            };
            let indent = captures[1].len();
            if entry_indent.is_none() {
                if indent == 0 {
                    break;
                }
                entry_indent = Some(indent);
            }
            if Some(indent) != entry_indent {
                next += 1;
                continue;
            }
            let Some(id) = yaml_scalar(&captures[2]) else {
                next += 1;
                continue;
            };
            let mut name = None;
            let mut name_fields = 0;
            let mut field = next + 1;
            while field < lines.len() {
                let field_line = lines[field];
                if !field_line.trim().is_empty() && leading_spaces(field_line) <= indent {
                    break;
                }
                if let Some(captures) = name_pattern.captures(field_line)
                    && captures[1].len() == indent + 2
                {
                    name_fields += 1;
                    name = yaml_scalar(&captures[2]);
                }
                field += 1;
            }
            entries.push(CordisEntry {
                id,
                valid: name.is_some() && name_fields == 1,
                name,
            });
            next += 1;
        }
    }
    entries
}

fn collect_skill_files(root: &Path, relative: &str) -> Vec<PathBuf> {
    fn walk(directory: &Path, depth: usize, results: &mut Vec<PathBuf>) {
        if depth > 5 {
            return;
        }
        let Ok(entries) = fs::read_dir(directory) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if file_type.is_dir() {
                walk(&path, depth + 1, results);
            } else if file_type.is_file()
                && path.file_name().and_then(|name| name.to_str()) == Some("SKILL.md")
            {
                results.push(path);
            }
        }
    }
    let start = root.join(relative);
    let mut results = Vec::new();
    walk(&start, 0, &mut results);
    results.sort();
    results
}

fn scan_for_tokens(root: &Path) -> Vec<PathBuf> {
    fn walk(directory: &Path, depth: usize, matches: &mut Vec<PathBuf>, pattern: &Regex) {
        if depth > 8 {
            return;
        }
        let ignored = [".git", "dist", "node_modules", "coverage", "templates"];
        let Ok(entries) = fs::read_dir(directory) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name();
            if ignored.iter().any(|ignored| name == *ignored) {
                continue;
            }
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if file_type.is_dir() {
                walk(&path, depth + 1, matches, pattern);
            } else if file_type.is_file()
                && fs::read_to_string(&path)
                    .ok()
                    .is_some_and(|content| pattern.is_match(&content))
            {
                matches.push(path);
            }
        }
    }
    let mut matches = Vec::new();
    walk(
        root,
        0,
        &mut matches,
        &Regex::new(r"\{\{[A-Z0-9_]+\}\}").unwrap(),
    );
    matches.sort();
    matches
}

fn check_product_skill(
    skill_file: &Path,
    content: &str,
    frontmatter: &HashMap<String, String>,
    errors: &mut Vec<String>,
    python_scripts: &mut Vec<PathBuf>,
    runtime: &str,
) {
    if frontmatter
        .get("description")
        .map_or(0, |value| value.chars().count())
        > 1024
    {
        errors.push(format!(
            "{}: description must be at most 1024 characters",
            skill_file.display()
        ));
    }
    if frontmatter
        .get("compatibility")
        .map_or(0, |value| value.chars().count())
        > 500
    {
        errors.push(format!(
            "{}: compatibility must be at most 500 characters",
            skill_file.display()
        ));
    }
    let skill_directory = skill_file.parent().unwrap();
    let reference_pattern =
        Regex::new(r"(?:scripts|references|assets)/[A-Za-z0-9][A-Za-z0-9._/-]*").unwrap();
    let mut references = HashSet::new();
    for found in reference_pattern.find_iter(content) {
        references.insert(
            found
                .as_str()
                .trim_end_matches(['.', ',', ';', ':', ')', ']']),
        );
    }
    for reference in references {
        if !skill_directory.join(reference).exists() {
            errors.push(format!(
                "{}: referenced path '{reference}' does not exist",
                skill_file.display()
            ));
        }
    }
    let scripts = skill_directory.join("scripts");
    let Ok(entries) = fs::read_dir(&scripts) else {
        return;
    };
    let names = entries
        .flatten()
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect::<Vec<_>>();
    let python = names
        .iter()
        .filter_map(|name| name.strip_suffix(".py"))
        .collect::<HashSet<_>>();
    let javascript = names
        .iter()
        .filter_map(|name| name.strip_suffix(".mjs"))
        .collect::<HashSet<_>>();
    if runtime == "npm" {
        for base in &python {
            if !javascript.contains(base) {
                errors.push(format!(
                    "{}: scripts/{base}.py is missing its scripts/{base}.mjs twin",
                    skill_file.display()
                ));
            }
        }
        for base in &javascript {
            if !python.contains(base) {
                errors.push(format!(
                    "{}: scripts/{base}.mjs is missing its scripts/{base}.py twin",
                    skill_file.display()
                ));
            }
        }
    }
    for name in names.iter().filter(|name| name.ends_with(".py")) {
        python_scripts.push(scripts.join(name));
    }
}

fn check_python_syntax(paths: &[PathBuf], errors: &mut Vec<String>) {
    for path in paths {
        let Ok(source) = fs::read_to_string(path) else {
            errors.push(format!("{}: could not read Python source", path.display()));
            continue;
        };
        if let Err(error) = parse_module(&source) {
            errors.push(format!(
                "{}: Python syntax check failed ({error})",
                path.display()
            ));
        }
    }
}

fn check_allowed_fields(
    object: &Map<String, Value>,
    allowed: &[&str],
    label: &str,
) -> Result<(), String> {
    let unknown = object
        .keys()
        .filter(|key| !allowed.contains(&key.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    if unknown.is_empty() {
        Ok(())
    } else {
        Err(format!("{label}: unknown fields: {}", unknown.join(", ")))
    }
}

fn required_string<'a>(
    object: &'a Map<String, Value>,
    key: &str,
    label: &str,
    min: usize,
    max: usize,
) -> Result<&'a str, String> {
    let value = object
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{label}.{key} must be a string"))?;
    let length = value.chars().count();
    if length < min || length > max {
        return Err(format!("{label}.{key} has invalid length"));
    }
    Ok(value)
}

fn scalar_matches(kind: &str, value: &Value) -> bool {
    match kind {
        "string" => value.is_string(),
        "number" => value.is_number(),
        "integer" => value.as_i64().is_some() || value.as_u64().is_some(),
        "boolean" => value.is_boolean(),
        "null" => value.is_null(),
        _ => false,
    }
}

fn validate_json_schema(schema: &Value, label: &str, input_root: bool) -> Result<(), String> {
    let object = schema
        .as_object()
        .ok_or_else(|| format!("{label} must be an object"))?;
    check_allowed_fields(
        object,
        &[
            "type",
            "oneOf",
            "properties",
            "required",
            "additionalProperties",
            "items",
            "enum",
            "const",
            "description",
            "title",
            "default",
            "examples",
        ],
        label,
    )?;
    let kind = object.get("type").and_then(Value::as_str);
    if let Some(kind) = kind {
        if !matches!(
            kind,
            "object" | "array" | "string" | "number" | "integer" | "boolean" | "null"
        ) {
            return Err(format!("{label}.type is invalid"));
        }
    } else if object.contains_key("type") {
        return Err(format!("{label}.type must be a string"));
    }
    if input_root && kind != Some("object") {
        return Err(format!("{label}: tool input must have type object"));
    }
    if input_root && object.get("additionalProperties") != Some(&Value::Bool(false)) {
        return Err(format!(
            "{label}: tool input must set additionalProperties to false"
        ));
    }

    let constraint_keys = [
        "type",
        "properties",
        "required",
        "additionalProperties",
        "items",
        "enum",
        "const",
    ]
    .into_iter()
    .filter(|key| object.contains_key(*key))
    .collect::<Vec<_>>();
    if let Some(one_of) = object.get("oneOf") {
        let branches = one_of
            .as_array()
            .ok_or_else(|| format!("{label}.oneOf must be an array"))?;
        if branches.len() < 2 {
            return Err(format!("{label}.oneOf: expected at least two branches"));
        }
        if !constraint_keys.is_empty() {
            return Err(format!(
                "{label}: oneOf cannot be combined with {}",
                constraint_keys.join(", ")
            ));
        }
        for (index, branch) in branches.iter().enumerate() {
            validate_json_schema(branch, &format!("{label}.oneOf[{index}]"), false)?;
        }
        return Ok(());
    }
    if ["properties", "required", "additionalProperties"]
        .iter()
        .any(|key| object.contains_key(*key))
        && kind != Some("object")
    {
        return Err(format!("{label}: object keywords require type object"));
    }
    if object.contains_key("items") && kind != Some("array") {
        return Err(format!("{label}: items requires type array"));
    }
    if (object.contains_key("enum") || object.contains_key("const"))
        && !matches!(
            kind,
            Some("string" | "number" | "integer" | "boolean" | "null")
        )
    {
        return Err(format!("{label}: enum and const require a scalar type"));
    }
    if let Some(properties) = object.get("properties") {
        let properties = properties
            .as_object()
            .ok_or_else(|| format!("{label}.properties must be an object"))?;
        for (name, child) in properties {
            validate_json_schema(child, &format!("{label}.properties.{name}"), false)?;
        }
    }
    if let Some(required) = object.get("required") {
        let required = required
            .as_array()
            .ok_or_else(|| format!("{label}.required must be an array"))?;
        let mut seen = HashSet::new();
        for value in required {
            let name = value
                .as_str()
                .ok_or_else(|| format!("{label}.required values must be strings"))?;
            if !seen.insert(name) {
                return Err(format!("{label}.required contains duplicate {name}"));
            }
            if !object
                .get("properties")
                .and_then(Value::as_object)
                .is_some_and(|properties| properties.contains_key(name))
            {
                return Err(format!(
                    "{label}.required: {name} is not declared in properties"
                ));
            }
        }
    }
    if let Some(additional) = object.get("additionalProperties")
        && !additional.is_boolean()
    {
        return Err(format!("{label}.additionalProperties must be boolean"));
    }
    if let Some(items) = object.get("items") {
        validate_json_schema(items, &format!("{label}.items"), false)?;
    }
    if let Some(values) = object.get("enum") {
        let values = values
            .as_array()
            .filter(|values| !values.is_empty())
            .ok_or_else(|| format!("{label}.enum must be a non-empty array"))?;
        let mut seen = HashSet::new();
        for value in values {
            if !kind.is_some_and(|kind| scalar_matches(kind, value)) {
                return Err(format!("{label}.enum: value does not match type"));
            }
            let serialized = serde_json::to_string(value).unwrap();
            if !seen.insert(serialized.clone()) {
                return Err(format!("{label}.enum: duplicate value {serialized}"));
            }
        }
    }
    if let Some(value) = object.get("const")
        && !kind.is_some_and(|kind| scalar_matches(kind, value))
    {
        return Err(format!("{label}.const: value does not match type"));
    }
    for key in ["description", "title"] {
        if object.get(key).is_some_and(|value| !value.is_string()) {
            return Err(format!("{label}.{key} must be a string"));
        }
    }
    Ok(())
}

fn validate_pep723(path: &Path) -> Result<(), String> {
    let source = fs::read_to_string(path).map_err(|error| error.to_string())?;
    let start = source.find("# /// script\n").ok_or_else(|| {
        format!(
            "python-uv entrypoint must contain a PEP 723 script block: {}",
            path.display()
        )
    })? + "# /// script\n".len();
    let rest = &source[start..];
    let end = rest.find("# ///").ok_or_else(|| {
        format!(
            "python-uv entrypoint must contain a PEP 723 script block: {}",
            path.display()
        )
    })?;
    let toml_source = rest[..end]
        .lines()
        .map(|line| {
            line.strip_prefix("# ")
                .or_else(|| line.strip_prefix('#'))
                .ok_or_else(|| "metadata lines must be comments".to_string())
        })
        .collect::<Result<Vec<_>, _>>()?
        .join("\n");
    let metadata = toml_source
        .parse::<toml::Value>()
        .map_err(|error| format!("python-uv entrypoint has invalid PEP 723 metadata: {error}"))?;
    let dependencies = metadata
        .get("dependencies")
        .and_then(toml::Value::as_array)
        .ok_or_else(|| "python-uv dependencies must be a TOML string array".to_string())?;
    let exact = Regex::new(r"^[A-Za-z0-9_.-]+==[^=\s]+$").unwrap();
    if dependencies
        .iter()
        .any(|value| value.as_str().is_none_or(|value| !exact.is_match(value)))
    {
        return Err("python-uv dependencies must use exact == pins".to_string());
    }
    Ok(())
}

fn validate_runtime_manifests(
    plugin_root: &Path,
    paths: &[PathBuf],
) -> Result<Vec<(String, PathBuf)>, String> {
    let canonical_root = fs::canonicalize(plugin_root).map_err(|error| {
        format!(
            "plugin root does not exist: {} ({error})",
            plugin_root.display()
        )
    })?;
    let manifest_name_pattern = Regex::new(r"^[a-z0-9]+(?:-[a-z0-9]+)*$").unwrap();
    let tool_name_pattern = Regex::new(r"^[a-z][a-z0-9_]{0,63}$").unwrap();
    let mut global_names = HashSet::new();
    let mut loaded = Vec::new();
    for requested in paths {
        let manifest_path = fs::canonicalize(requested).map_err(|error| {
            format!("manifest does not exist: {} ({error})", requested.display())
        })?;
        if !path_utils::is_contained(&canonical_root, &manifest_path) {
            return Err(format!(
                "manifest escapes plugin root: {}",
                requested.display()
            ));
        }
        if manifest_path.file_name().and_then(|name| name.to_str()) != Some("skill-runtime.json") {
            return Err(format!(
                "runtime manifest must be named skill-runtime.json: {}",
                requested.display()
            ));
        }
        let value: Value =
            serde_json::from_str(&fs::read_to_string(&manifest_path).map_err(|error| {
                format!("could not parse {} ({error})", manifest_path.display())
            })?)
            .map_err(|error| format!("could not parse {} ({error})", manifest_path.display()))?;
        let manifest = value
            .as_object()
            .ok_or_else(|| format!("{}: manifest must be an object", manifest_path.display()))?;
        check_allowed_fields(
            manifest,
            &["$schema", "manifestVersion", "name", "tools"],
            "manifest",
        )?;
        if manifest
            .get("$schema")
            .is_some_and(|value| !value.is_string())
        {
            return Err("manifest.$schema must be a string".to_string());
        }
        if manifest.get("manifestVersion").and_then(Value::as_u64) != Some(1) {
            return Err("manifest.manifestVersion must equal 1".to_string());
        }
        let manifest_name = required_string(manifest, "name", "manifest", 1, 80)?;
        if !manifest_name_pattern.is_match(manifest_name) {
            return Err("manifest.name is invalid".to_string());
        }
        let tools = manifest
            .get("tools")
            .and_then(Value::as_array)
            .filter(|tools| !tools.is_empty())
            .ok_or_else(|| "manifest.tools must be a non-empty array".to_string())?;
        let mut local_names = HashSet::new();
        let skill_root = manifest_path.parent().unwrap();
        for (index, tool_value) in tools.iter().enumerate() {
            let label = format!("tools[{index}]");
            let tool = tool_value
                .as_object()
                .ok_or_else(|| format!("{label} must be an object"))?;
            check_allowed_fields(
                tool,
                &[
                    "title",
                    "name",
                    "description",
                    "inputSchema",
                    "outputSchema",
                    "entrypoint",
                    "capabilities",
                    "annotations",
                    "limits",
                ],
                &label,
            )?;
            required_string(tool, "title", &label, 1, 120)?;
            let tool_name = required_string(tool, "name", &label, 1, 64)?;
            if !tool_name_pattern.is_match(tool_name) {
                return Err(format!("{label}.name is invalid"));
            }
            required_string(tool, "description", &label, 1, 1024)?;
            if !local_names.insert(tool_name.to_string()) {
                return Err(format!(
                    "{}: duplicate tool name {tool_name}",
                    manifest_path.display()
                ));
            }
            if !global_names.insert(tool_name.to_string()) {
                return Err(format!("duplicate tool name across manifests: {tool_name}"));
            }
            validate_json_schema(
                tool.get("inputSchema")
                    .ok_or_else(|| format!("{label}.inputSchema is required"))?,
                &format!("{tool_name}.inputSchema"),
                true,
            )?;
            if let Some(output) = tool.get("outputSchema") {
                validate_json_schema(output, &format!("{tool_name}.outputSchema"), false)?;
            }
            let entrypoint = tool
                .get("entrypoint")
                .and_then(Value::as_object)
                .ok_or_else(|| format!("{label}.entrypoint must be an object"))?;
            check_allowed_fields(entrypoint, &["engine", "path", "args"], "entrypoint")?;
            let engine = required_string(entrypoint, "engine", "entrypoint", 1, usize::MAX)?;
            if !matches!(engine, "node" | "python" | "python-uv" | "wasmtime") {
                return Err(format!("{tool_name}.entrypoint.engine is invalid"));
            }
            let entrypoint_path = required_string(entrypoint, "path", "entrypoint", 1, usize::MAX)?;
            if entrypoint_path.contains('\0') {
                return Err(format!("{tool_name}.entrypoint.path contains a null byte"));
            }
            if let Some(args) = entrypoint.get("args") {
                let args = args
                    .as_array()
                    .filter(|args| args.len() <= 32)
                    .ok_or_else(|| {
                        "entrypoint.args must be an array with at most 32 items".to_string()
                    })?;
                if args.iter().any(|arg| !arg.is_string()) {
                    return Err("entrypoint.args must contain strings".to_string());
                }
            }
            let resolved = path_utils::absolute_normalized(&skill_root.join(entrypoint_path))?;
            if !path_utils::is_contained(skill_root, &resolved) {
                return Err(format!(
                    "{tool_name}.entrypoint.path escapes the skill root: {entrypoint_path}"
                ));
            }
            let canonical_entrypoint = fs::canonicalize(&resolved).map_err(|_| {
                format!("{tool_name}.entrypoint.path does not exist: {entrypoint_path}")
            })?;
            let canonical_skill =
                fs::canonicalize(skill_root).map_err(|error| error.to_string())?;
            if !path_utils::is_contained(&canonical_skill, &canonical_entrypoint) {
                return Err(format!(
                    "{tool_name}.entrypoint.path escapes the skill root: {entrypoint_path}"
                ));
            }
            let valid_extension = match engine {
                "node" => ["js", "mjs", "cjs"].iter().any(|extension| {
                    canonical_entrypoint
                        .extension()
                        .and_then(|value| value.to_str())
                        == Some(extension)
                }),
                "python" | "python-uv" => {
                    canonical_entrypoint
                        .extension()
                        .and_then(|value| value.to_str())
                        == Some("py")
                }
                "wasmtime" => {
                    canonical_entrypoint
                        .extension()
                        .and_then(|value| value.to_str())
                        == Some("wasm")
                }
                _ => false,
            };
            if !valid_extension {
                return Err(format!(
                    "{tool_name}.entrypoint.path does not match engine {engine}"
                ));
            }
            if engine == "python-uv" {
                validate_pep723(&canonical_entrypoint)?;
            }
            let capabilities = tool
                .get("capabilities")
                .map(|value| {
                    value
                        .as_array()
                        .ok_or_else(|| format!("{label}.capabilities must be an array"))
                })
                .transpose()?;
            if let Some(capabilities) = capabilities {
                let mut seen = HashSet::new();
                for capability in capabilities {
                    let capability = capability
                        .as_str()
                        .ok_or_else(|| format!("{label}.capabilities must contain strings"))?;
                    if !matches!(
                        capability,
                        "filesystem-read" | "filesystem-write" | "network" | "process"
                    ) {
                        return Err(format!("{label}.capabilities contains an invalid value"));
                    }
                    if !seen.insert(capability) {
                        return Err(format!("{label}.capabilities contains duplicate values"));
                    }
                    if engine == "wasmtime" && capability != "filesystem-read" {
                        return Err(format!(
                            "{tool_name}: wasmtime entrypoints cannot request write, network, or process capabilities"
                        ));
                    }
                }
            }
            if let Some(annotations) = tool.get("annotations") {
                let annotations = annotations
                    .as_object()
                    .ok_or_else(|| format!("{label}.annotations must be an object"))?;
                let fields = [
                    "readOnlyHint",
                    "destructiveHint",
                    "idempotentHint",
                    "openWorldHint",
                ];
                check_allowed_fields(annotations, &fields, "annotations")?;
                if fields
                    .iter()
                    .any(|field| !annotations.get(*field).is_some_and(Value::is_boolean))
                {
                    return Err(format!("{label}.annotations requires four boolean hints"));
                }
            }
            if let Some(limits) = tool.get("limits") {
                let limits = limits
                    .as_object()
                    .ok_or_else(|| format!("{label}.limits must be an object"))?;
                check_allowed_fields(
                    limits,
                    &["timeoutMs", "maxInputBytes", "maxOutputBytes"],
                    "limits",
                )?;
                for (field, maximum) in [
                    ("timeoutMs", 600_000_u64),
                    ("maxInputBytes", 16_777_216),
                    ("maxOutputBytes", 16_777_216),
                ] {
                    if let Some(value) = limits.get(field) {
                        let value = value
                            .as_u64()
                            .ok_or_else(|| format!("{label}.limits.{field} must be an integer"))?;
                        if value == 0 || value > maximum {
                            return Err(format!("{label}.limits.{field} is out of range"));
                        }
                    }
                }
            }
        }
        loaded.push((manifest_name.to_string(), manifest_path));
    }
    Ok(loaded)
}

pub fn validate_project(project_root: &Path, external: bool) -> ValidationResult {
    let root = path_utils::absolute_normalized(project_root)
        .unwrap_or_else(|_| project_root.to_path_buf());
    let root_string = root.to_string_lossy().into_owned();
    let mut errors = Vec::new();
    let warnings = Vec::new();
    let layout_path = root.join("alchemy.json");
    let layout_value = if layout_path.exists() {
        match read_json(&layout_path, &mut errors) {
            Some(value) => value,
            None => {
                return ValidationResult {
                    root: root_string,
                    errors,
                    warnings,
                };
            }
        }
    } else {
        Value::Object(Map::new())
    };
    let Some(layout) = object(&layout_value, "alchemy.json", &mut errors) else {
        return ValidationResult {
            root: root_string,
            errors,
            warnings,
        };
    };
    let known = [
        "pluginRoot",
        "opencodeExport",
        "runtime",
        "$schema",
        "template",
        "generator",
        "generatorVersion",
        "createdAt",
    ];
    let unknown = layout
        .keys()
        .filter(|key| !known.contains(&key.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    if !unknown.is_empty() {
        errors.push(format!(
            "alchemy.json has unknown fields: {}",
            unknown.join(", ")
        ));
    }
    if layout
        .get("$schema")
        .is_some_and(|value| !value.is_string())
    {
        errors.push("alchemy.json $schema must be a string".to_string());
    }
    let template_pattern = Regex::new(r"^v[0-9]+\.[0-9]+\.[0-9]+(?:-[0-9A-Za-z.-]+)?$").unwrap();
    if let Some(value) = layout.get("template")
        && value
            .as_str()
            .is_none_or(|value| !template_pattern.is_match(value))
    {
        errors.push("alchemy.json template must be a 'vX.Y.Z' version".to_string());
    }
    if layout
        .get("generator")
        .is_some_and(|value| value.as_str() != Some("harness-alchemist"))
    {
        errors.push("alchemy.json generator must be 'harness-alchemist'".to_string());
    }
    let version_pattern = Regex::new(r"^[0-9]+\.[0-9]+\.[0-9]+(?:-[0-9A-Za-z.-]+)?$").unwrap();
    if let Some(value) = layout.get("generatorVersion")
        && value
            .as_str()
            .is_none_or(|value| !version_pattern.is_match(value))
    {
        errors.push("alchemy.json generatorVersion must be an 'X.Y.Z' version".to_string());
    }
    if let Some(value) = layout.get("createdAt")
        && value.as_str().is_none_or(|value| !valid_created_at(value))
    {
        errors.push("alchemy.json createdAt must be an ISO 8601 timestamp".to_string());
    }
    let runtime = match layout.get("runtime") {
        None => "npm",
        Some(value) => value.as_str().unwrap_or(""),
    };
    if !RUNTIME_MODES.contains(&runtime) {
        errors.push(format!(
            "alchemy.json runtime must be one of: {}",
            RUNTIME_MODES.join(", ")
        ));
    }
    let plugin_root_relative = match layout.get("pluginRoot") {
        None => ".",
        Some(value) => value.as_str().unwrap_or(""),
    };
    if plugin_root_relative.trim().is_empty() {
        errors.push("alchemy.json pluginRoot must be a non-empty relative path".to_string());
    }
    let plugin_root = path_utils::absolute_normalized(&root.join(plugin_root_relative))
        .unwrap_or_else(|_| root.clone());
    if !path_utils::is_contained(&root, &plugin_root) {
        errors.push("alchemy.json pluginRoot escapes the project root".to_string());
    } else if plugin_root.exists()
        && let (Ok(canonical_root), Ok(canonical_plugin)) =
            (fs::canonicalize(&root), fs::canonicalize(&plugin_root))
        && !path_utils::is_contained(&canonical_root, &canonical_plugin)
    {
        errors.push(
            "alchemy.json pluginRoot resolves through a symlink outside the project root"
                .to_string(),
        );
    }
    let opencode_export = match layout.get("opencodeExport") {
        None => ".",
        Some(value) => value.as_str().unwrap_or(""),
    };
    if !matches!(opencode_export, "." | "./server") {
        errors.push("alchemy.json opencodeExport must be '.' or './server'".to_string());
    }
    if opencode_export != "." && runtime != "npm" {
        errors.push("alchemy.json opencodeExport requires runtime 'npm'".to_string());
    }
    if !errors.is_empty() {
        return ValidationResult {
            root: root_string,
            errors,
            warnings,
        };
    }

    for file in PROJECT_REQUIRED_FILES {
        require_path(&root, file, &mut errors);
    }
    if runtime == "npm" {
        require_path(&root, ".github/workflows/npm-publish.yml", &mut errors);
    }
    for file in PLUGIN_REQUIRED_FILES {
        require_path(&plugin_root, file, &mut errors);
    }
    if runtime == "npm" {
        for file in NPM_PLUGIN_REQUIRED_FILES {
            require_path(&plugin_root, file, &mut errors);
        }
    }
    if !errors.is_empty() {
        return ValidationResult {
            root: root_string,
            errors,
            warnings,
        };
    }

    let Some(claude_plugin) =
        read_json(&plugin_root.join(".claude-plugin/plugin.json"), &mut errors)
    else {
        return ValidationResult {
            root: root_string,
            errors,
            warnings,
        };
    };
    let Some(claude_marketplace) =
        read_json(&root.join(".claude-plugin/marketplace.json"), &mut errors)
    else {
        return ValidationResult {
            root: root_string,
            errors,
            warnings,
        };
    };
    let Some(codex_plugin) = read_json(&plugin_root.join(".codex-plugin/plugin.json"), &mut errors)
    else {
        return ValidationResult {
            root: root_string,
            errors,
            warnings,
        };
    };
    let Some(codex_marketplace) =
        read_json(&root.join(".agents/plugins/marketplace.json"), &mut errors)
    else {
        return ValidationResult {
            root: root_string,
            errors,
            warnings,
        };
    };
    let Some(agent_plugin) = read_json(&plugin_root.join("plugin.json"), &mut errors) else {
        return ValidationResult {
            root: root_string,
            errors,
            warnings,
        };
    };
    let Some(agent_mcp) = read_json(&plugin_root.join("mcp.json"), &mut errors) else {
        return ValidationResult {
            root: root_string,
            errors,
            warnings,
        };
    };

    let plugin_name = string(&claude_plugin, "name").unwrap_or("");
    let name_pattern = Regex::new(r"^[a-z0-9]+(?:-[a-z0-9]+)*$").unwrap();
    if !name_pattern.is_match(plugin_name) {
        errors.push(
            "Plugin name must contain lowercase letters, digits, and single hyphens".to_string(),
        );
    }
    if plugin_name.chars().count() > 56 {
        errors.push("Plugin name must be at most 56 characters".to_string());
    }
    for (label, value) in [
        ("Codex plugin", string(&codex_plugin, "name")),
        ("Agent plugin", string(&agent_plugin, "name")),
    ] {
        if value != Some(plugin_name) {
            errors.push(format!(
                "{label} name '{}' does not match '{plugin_name}'",
                value.unwrap_or("undefined")
            ));
        }
    }
    if string(&agent_plugin, "$schema") != Some(AGENT_PLUGIN_SCHEMA) {
        errors.push(format!(
            "plugin.json $schema must be '{AGENT_PLUGIN_SCHEMA}'"
        ));
    }
    if string(&agent_mcp, "$schema") != Some(AGENT_PLUGIN_MCP_SCHEMA) {
        errors.push(format!(
            "mcp.json $schema must be '{AGENT_PLUGIN_MCP_SCHEMA}'"
        ));
    }
    let expected_args = vec![
        "-y".to_string(),
        format!("{SKILL_RUNTIME_PACKAGE}@{SKILL_RUNTIME_VERSION}"),
        "mcp".to_string(),
        "--root".to_string(),
        "${PLUGIN_ROOT}".to_string(),
        "--manifest".to_string(),
        format!("skills/{plugin_name}/skill-runtime.json"),
    ];
    let mcp = agent_mcp
        .get("mcpServers")
        .and_then(|value| value.get(plugin_name));
    let actual_args = mcp
        .and_then(|value| value.get("args"))
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect::<Vec<_>>()
        });
    if mcp.and_then(|value| string(value, "type")) != Some("stdio")
        || mcp.and_then(|value| string(value, "command")) != Some("npx")
        || mcp.and_then(|value| string(value, "cwd")) != Some("${PLUGIN_ROOT}")
        || actual_args.as_ref() != Some(&expected_args)
    {
        errors.push(format!("mcp.json must configure '{plugin_name}' through {SKILL_RUNTIME_PACKAGE}@{SKILL_RUNTIME_VERSION}"));
    }

    if runtime == "skills" {
        if codex_plugin.get("version") != claude_plugin.get("version") {
            errors.push("Codex plugin version does not match the Claude plugin".to_string());
        }
        if codex_plugin.get("description") != claude_plugin.get("description") {
            errors.push("Codex plugin description does not match the Claude plugin".to_string());
        }
        if agent_plugin.get("version") != claude_plugin.get("version") {
            errors.push("Agent plugin version does not match the Claude plugin".to_string());
        }
        if agent_plugin.get("description") != claude_plugin.get("description") {
            errors.push("Agent plugin description does not match the Claude plugin".to_string());
        }
    } else {
        let Some(package) = read_json(&plugin_root.join("package.json"), &mut errors) else {
            return ValidationResult {
                root: root_string,
                errors,
                warnings,
            };
        };
        let package_name = string(&package, "name").unwrap_or("");
        if package_name.rsplit('/').next() != Some(plugin_name) {
            errors.push(format!(
                "npm package basename '{}' must match plugin name '{plugin_name}'",
                package_name.rsplit('/').next().unwrap_or("undefined")
            ));
        }
        for (label, manifest) in [
            ("Claude plugin", &claude_plugin),
            ("Codex plugin", &codex_plugin),
        ] {
            if manifest.get("version") != package.get("version") {
                errors.push(format!("{label} version does not match package.json"));
            }
            if manifest.get("description") != package.get("description") {
                errors.push(format!("{label} description does not match package.json"));
            }
        }
        if agent_plugin.get("version") != package.get("version") {
            errors.push("Agent plugin version does not match package.json".to_string());
        }
        if agent_plugin.get("description") != package.get("description") {
            errors.push("Agent plugin description does not match package.json".to_string());
        }
        if agent_plugin.get("license") != package.get("license") {
            errors.push("Agent plugin license does not match package.json".to_string());
        }
        if string(&package, "type") != Some("module") {
            errors.push("package.json type must be 'module'".to_string());
        }
        if package
            .get("dependencies")
            .and_then(|value| string(value, SKILL_RUNTIME_PACKAGE))
            != Some(SKILL_RUNTIME_VERSION)
        {
            errors.push(format!("package.json dependencies must pin '{SKILL_RUNTIME_PACKAGE}' to '{SKILL_RUNTIME_VERSION}'"));
        }
        if opencode_export == "."
            && package
                .get("engines")
                .and_then(|value| string(value, "node"))
                != Some(">=22.20.0")
        {
            errors.push("package.json engines.node must be '>=22.20.0'".to_string());
        }
        let empty = Value::Object(Map::new());
        let exports = package.get("exports").unwrap_or(&empty);
        for (key, field, expected) in [
            (opencode_export, "import", "./dist/opencode.js"),
            (opencode_export, "types", "./dist/opencode.d.ts"),
            ("./deepseek", "import", "./dist/deepseek.js"),
            ("./deepseek", "types", "./dist/deepseek.d.ts"),
            ("./cordis.patch.yml", "import", "./cordis.patch.yml"),
        ] {
            if export_target(exports, key, field) != Some(expected) {
                errors.push(format!(
                    "package.json export '{key}' {field} target must be '{expected}'"
                ));
            }
        }
        let files = package.get("files").unwrap_or(&Value::Null);
        for entry in [
            "dist",
            "skills",
            "alchemy.json",
            "cordis.patch.yml",
            ".claude-plugin/plugin.json",
            ".codex-plugin/plugin.json",
            "mcp.json",
            "plugin.json",
        ] {
            if !array_has(files, entry) {
                errors.push(format!("package.json files is missing '{entry}'"));
            }
        }
        if package
            .get("dsh")
            .and_then(|value| value.get("bundle"))
            .and_then(|value| string(value, "patch"))
            != Some("./cordis.patch.yml")
        {
            errors.push("package.json dsh.bundle.patch must be './cordis.patch.yml'".to_string());
        }
        let patch_path = plugin_root.join("cordis.patch.yml");
        let patch = fs::read_to_string(&patch_path).unwrap_or_default();
        let matching = parse_cordis_entries(&patch)
            .into_iter()
            .filter(|entry| entry.id == plugin_name)
            .collect::<Vec<_>>();
        if matching.len() != 1 || !matching[0].valid {
            errors.push(format!(
                "cordis.patch.yml must contain a valid insert entry for '{plugin_name}'"
            ));
        } else {
            let expected = format!("{package_name}/deepseek");
            if matching[0].name.as_deref() != Some(&expected) {
                errors.push(format!(
                    "cordis.patch.yml entry '{plugin_name}' must load '{expected}'"
                ));
            }
        }
    }

    require_manifest_path(
        &plugin_root,
        string(&claude_plugin, "skills"),
        "Claude skills",
        &mut errors,
    );
    require_manifest_path(
        &plugin_root,
        string(&codex_plugin, "skills"),
        "Codex skills",
        &mut errors,
    );
    let marketplace_source = if plugin_root == root {
        "./".to_string()
    } else {
        format!(
            "./{}",
            plugin_root_relative
                .trim_start_matches("./")
                .replace('\\', "/")
        )
    };
    let claude_entry = claude_marketplace
        .get("plugins")
        .and_then(Value::as_array)
        .and_then(|plugins| {
            plugins
                .iter()
                .find(|entry| string(entry, "name") == Some(plugin_name))
        });
    match claude_entry {
        None => errors.push("Claude marketplace is missing the plugin entry".to_string()),
        Some(entry) if string(entry, "source") != Some(&marketplace_source) => errors.push(
            format!("Claude marketplace source must be '{marketplace_source}'"),
        ),
        _ => {}
    }
    let codex_entry = codex_marketplace
        .get("plugins")
        .and_then(Value::as_array)
        .and_then(|plugins| {
            plugins
                .iter()
                .find(|entry| string(entry, "name") == Some(plugin_name))
        });
    match codex_entry {
        None => errors.push("Codex marketplace is missing the plugin entry".to_string()),
        Some(entry) => {
            let source = entry.get("source");
            if source.and_then(|value| string(value, "source")) != Some("local")
                || source.and_then(|value| string(value, "path")) != Some(&marketplace_source)
            {
                errors.push(format!(
                    "Codex marketplace source must be a local '{marketplace_source}' path"
                ));
            }
            let policy = entry.get("policy");
            if policy.and_then(|value| value.get("installation")).is_none()
                || policy
                    .and_then(|value| value.get("authentication"))
                    .is_none()
            {
                errors.push(
                    "Codex marketplace entry requires installation and authentication policies"
                        .to_string(),
                );
            }
            if entry.get("category").is_none() {
                errors.push("Codex marketplace entry requires a category".to_string());
            }
        }
    }

    let maintenance_skills = collect_skill_files(&root, ".agents/skills");
    let product_skills = collect_skill_files(&plugin_root, "skills");
    if maintenance_skills.len() + product_skills.len() < 2 {
        errors.push("Expected a shared skill and a project development skill".to_string());
    }
    let product_set = product_skills.iter().cloned().collect::<HashSet<_>>();
    let mut python_scripts = Vec::new();
    let mut runtime_manifests = Vec::new();
    for skill_file in product_skills.iter().chain(maintenance_skills.iter()) {
        let content = fs::read_to_string(skill_file).unwrap_or_default();
        let frontmatter = parse_frontmatter(&content);
        let directory_name = skill_file
            .parent()
            .and_then(Path::file_name)
            .and_then(|name| name.to_str())
            .unwrap_or("");
        match frontmatter {
            None => errors.push(format!(
                "{}: missing YAML frontmatter",
                skill_file.display()
            )),
            Some(frontmatter) => {
                if frontmatter.get("name").map(String::as_str) != Some(directory_name) {
                    errors.push(format!(
                        "{}: frontmatter name must match directory '{directory_name}'",
                        skill_file.display()
                    ));
                }
                if frontmatter.get("description").is_none_or(String::is_empty) {
                    errors.push(format!("{}: description is required", skill_file.display()));
                }
                let name = frontmatter.get("name").map(String::as_str).unwrap_or("");
                if !name_pattern.is_match(name) || name.chars().count() > 64 {
                    errors.push(format!(
                        "{}: invalid Agent Skill name",
                        skill_file.display()
                    ));
                }
                if product_set.contains(skill_file) {
                    let manifest = skill_file.parent().unwrap().join("skill-runtime.json");
                    if !manifest.exists() && (runtime == "npm" || directory_name == plugin_name) {
                        errors.push(format!(
                            "{}: missing skill-runtime.json",
                            skill_file.display()
                        ));
                    } else if manifest.exists() {
                        runtime_manifests.push(manifest);
                    }
                    check_product_skill(
                        skill_file,
                        &content,
                        &frontmatter,
                        &mut errors,
                        &mut python_scripts,
                        runtime,
                    );
                }
            }
        }
    }
    if !runtime_manifests.is_empty() {
        match validate_runtime_manifests(&plugin_root, &runtime_manifests) {
            Ok(manifests) => {
                for (manifest_name, path) in manifests {
                    let directory = path
                        .parent()
                        .and_then(Path::file_name)
                        .and_then(|name| name.to_str())
                        .unwrap_or("");
                    if manifest_name != directory {
                        errors.push(format!(
                            "{}: manifest name must match skill directory '{directory}'",
                            path.display()
                        ));
                    }
                }
            }
            Err(error) => errors.push(format!("Skill runtime validation failed: {error}")),
        }
    }
    check_python_syntax(&python_scripts, &mut errors);
    for path in scan_for_tokens(&root) {
        errors.push(format!("{}: unresolved scaffold token", path.display()));
    }
    if external {
        let result = process::run(
            "claude",
            [
                "plugin",
                "validate",
                plugin_root.to_string_lossy().as_ref(),
                "--strict",
            ],
            process::RunOptions {
                timeout: Some(Duration::from_secs(300)),
                ..process::RunOptions::default()
            },
        );
        if result.missing {
            // This warning remains non-fatal because the external validator is optional.
            let mut result_warnings = warnings;
            result_warnings.push("Claude CLI not found; skipped external validation".to_string());
            return ValidationResult {
                root: root_string,
                errors,
                warnings: result_warnings,
            };
        }
        if !result.ok {
            errors.push(format!(
                "Claude plugin validation failed: {}",
                if result.stderr.trim().is_empty() {
                    result.stdout.trim()
                } else {
                    result.stderr.trim()
                }
            ));
        }
    }
    ValidationResult {
        root: root_string,
        errors,
        warnings,
    }
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
    let root = if let Some(project) = args.project {
        path_utils::absolute_normalized(Path::new(&project)).ok()
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
    let result = validate_project(&root, args.external);
    if args.json {
        let output = ValidationOutput {
            valid: result.errors.is_empty(),
            root: &result.root,
            errors: &result.errors,
            warnings: &result.warnings,
        };
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
    } else if result.errors.is_empty() {
        println!("Validated universal plugin scaffold at {}", result.root);
        for warning in &result.warnings {
            eprintln!("Warning: {warning}");
        }
    } else {
        eprintln!("Validation failed for {}:", result.root);
        for error in &result.errors {
            eprintln!("- {error}");
        }
        for warning in &result.warnings {
            eprintln!("Warning: {warning}");
        }
    }
    i32::from(!result.errors.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_portable_schema_keywords() {
        let schema = serde_json::json!({
            "type": "object",
            "properties": {},
            "additionalProperties": false,
            "minimum": 1
        });
        assert!(validate_json_schema(&schema, "input", true).is_err());
    }

    #[test]
    fn parses_valid_cordis_insert() {
        let entries =
            parse_cordis_entries("- insert:\n    - id: demo\n      name: 'demo/deepseek'\n");
        assert_eq!(entries.len(), 1);
        assert!(entries[0].valid);
        assert_eq!(entries[0].name.as_deref(), Some("demo/deepseek"));
    }

    #[test]
    fn accepts_javascript_compatible_iso_dates() {
        assert!(valid_created_at("2024-01-01"));
        assert!(valid_created_at("2024-01-01T12:30"));
        assert!(valid_created_at("2024-01-01T12:30Z"));
        assert!(valid_created_at("2024-01-01T12:30:45"));
        assert!(valid_created_at("2024-01-01T12:30:45.123Z"));
        assert!(!valid_created_at("not-a-date"));
    }
}
