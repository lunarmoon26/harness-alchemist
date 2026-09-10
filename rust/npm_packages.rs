use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use serde_json::{Map, Value};

use crate::path_utils;

struct Target {
    triple: &'static str,
    package: &'static str,
    os: &'static str,
    cpu: &'static str,
    executable: &'static str,
}

const TARGETS: &[Target] = &[
    Target {
        triple: "aarch64-apple-darwin",
        package: "@lunarmoon26/harness-alchemist-darwin-arm64",
        os: "darwin",
        cpu: "arm64",
        executable: "harness-alchemist",
    },
    Target {
        triple: "x86_64-apple-darwin",
        package: "@lunarmoon26/harness-alchemist-darwin-x64",
        os: "darwin",
        cpu: "x64",
        executable: "harness-alchemist",
    },
    Target {
        triple: "aarch64-unknown-linux-musl",
        package: "@lunarmoon26/harness-alchemist-linux-arm64",
        os: "linux",
        cpu: "arm64",
        executable: "harness-alchemist",
    },
    Target {
        triple: "x86_64-unknown-linux-musl",
        package: "@lunarmoon26/harness-alchemist-linux-x64",
        os: "linux",
        cpu: "x64",
        executable: "harness-alchemist",
    },
    Target {
        triple: "aarch64-pc-windows-msvc",
        package: "@lunarmoon26/harness-alchemist-win32-arm64",
        os: "win32",
        cpu: "arm64",
        executable: "harness-alchemist.exe",
    },
    Target {
        triple: "x86_64-pc-windows-msvc",
        package: "@lunarmoon26/harness-alchemist-win32-x64",
        os: "win32",
        cpu: "x64",
        executable: "harness-alchemist.exe",
    },
];

struct Args {
    output: PathBuf,
    artifacts: Option<PathBuf>,
    host_binary: Option<PathBuf>,
    target: String,
    require_all: bool,
}

fn usage() -> &'static str {
    "Usage: harness-alchemist __stage-native-packages --output <directory> [options]\n\n\
Options:\n\
  --artifacts <directory>   Target-named directories containing release binaries.\n\
  --host-binary <path>      Stage one locally built binary.\n\
  --target <triple>         Target triple for --host-binary.\n\
  --require-all             Fail unless all supported targets are present."
}

fn parse_args(args: &[String]) -> Result<Args, String> {
    let mut output = None;
    let mut artifacts = None;
    let mut host_binary = None;
    let mut target = env!("HARNESS_ALCHEMIST_TARGET").to_string();
    let mut require_all = false;
    let mut index = 0;
    while index < args.len() {
        let argument = &args[index];
        if argument == "--require-all" {
            require_all = true;
            index += 1;
            continue;
        }
        let destination = match argument.as_str() {
            "--output" => "output",
            "--artifacts" => "artifacts",
            "--host-binary" => "host-binary",
            "--target" => "target",
            _ => return Err(format!("Unknown option: {argument}")),
        };
        let value = args
            .get(index + 1)
            .filter(|value| !value.starts_with("--"))
            .ok_or_else(|| format!("{argument} requires a value"))?;
        match destination {
            "output" => output = Some(PathBuf::from(value)),
            "artifacts" => artifacts = Some(PathBuf::from(value)),
            "host-binary" => host_binary = Some(PathBuf::from(value)),
            "target" => target = value.clone(),
            _ => unreachable!(),
        }
        index += 2;
    }
    Ok(Args {
        output: output.ok_or_else(|| "--output is required".to_string())?,
        artifacts,
        host_binary,
        target,
        require_all,
    })
}

fn copy_path(source: &Path, destination: &Path) -> Result<(), String> {
    let metadata =
        fs::symlink_metadata(source).map_err(|error| format!("{}: {error}", source.display()))?;
    if metadata.is_dir() {
        fs::create_dir_all(destination).map_err(|error| error.to_string())?;
        for entry in fs::read_dir(source).map_err(|error| error.to_string())? {
            let entry = entry.map_err(|error| error.to_string())?;
            copy_path(&entry.path(), &destination.join(entry.file_name()))?;
        }
    } else if metadata.is_file() {
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        fs::copy(source, destination).map_err(|error| error.to_string())?;
    } else {
        return Err(format!(
            "Unsupported package payload path: {}",
            source.display()
        ));
    }
    Ok(())
}

fn write_json(path: &Path, value: &Value) -> Result<(), String> {
    fs::write(
        path,
        format!(
            "{}\n",
            serde_json::to_string_pretty(value).map_err(|error| error.to_string())?
        ),
    )
    .map_err(|error| error.to_string())
}

fn package_directory(output: &Path, package: &str) -> PathBuf {
    output
        .join("platform")
        .join(package.rsplit('/').next().unwrap_or(package))
}

#[cfg(unix)]
fn make_executable(path: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = fs::metadata(path)
        .map_err(|error| error.to_string())?
        .permissions();
    permissions.set_mode(permissions.mode() | 0o111);
    fs::set_permissions(path, permissions).map_err(|error| error.to_string())
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) -> Result<(), String> {
    Ok(())
}

fn distribution_target(target: &str) -> &str {
    match target {
        "aarch64-unknown-linux-gnu" => "aarch64-unknown-linux-musl",
        "x86_64-unknown-linux-gnu" => "x86_64-unknown-linux-musl",
        target => target,
    }
}

fn stage_platform(
    root: &Path,
    output: &Path,
    package_json: &Value,
    target: &Target,
    binary: &Path,
) -> Result<(), String> {
    let directory = package_directory(output, target.package);
    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    let staged_binary = directory.join(target.executable);
    copy_path(binary, &staged_binary)?;
    if target.os != "win32" {
        make_executable(&staged_binary)?;
    }
    copy_path(&root.join("LICENSE"), &directory.join("LICENSE"))?;
    let manifest = serde_json::json!({
        "name": target.package,
        "version": package_json["version"],
        "description": format!("Native {} {} binary for Harness Alchemist.", target.os, target.cpu),
        "license": package_json["license"],
        "repository": package_json["repository"],
        "publishConfig": { "access": "public" },
        "preferUnplugged": true,
        "os": [target.os],
        "cpu": [target.cpu],
        "files": [target.executable, "LICENSE"]
    });
    write_json(&directory.join("package.json"), &manifest)
}

fn stage_main(root: &Path, output: &Path, package_json: &Value) -> Result<(), String> {
    let directory = output.join("main");
    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    let files = package_json["files"]
        .as_array()
        .ok_or_else(|| "package.json files must be an array".to_string())?;
    for relative in files {
        let relative = relative
            .as_str()
            .ok_or_else(|| "package.json files entries must be strings".to_string())?;
        copy_path(&root.join(relative), &directory.join(relative))?;
    }

    let mut manifest = package_json.clone();
    let object = manifest
        .as_object_mut()
        .ok_or_else(|| "package.json must contain an object".to_string())?;
    object.remove("devDependencies");
    object.remove("scripts");
    object.insert("preferUnplugged".to_string(), Value::Bool(true));
    object.insert(
        "publishConfig".to_string(),
        serde_json::json!({ "access": "public" }),
    );
    let optional = TARGETS
        .iter()
        .map(|target| {
            (
                target.package.to_string(),
                Value::String(
                    package_json["version"]
                        .as_str()
                        .unwrap_or_default()
                        .to_string(),
                ),
            )
        })
        .collect::<Map<_, _>>();
    object.insert("optionalDependencies".to_string(), Value::Object(optional));
    write_json(&directory.join("package.json"), &manifest)
}

fn stage(args: Args) -> Result<(), String> {
    let root =
        path_utils::package_root().ok_or_else(|| "Could not find package root".to_string())?;
    let output = path_utils::absolute_normalized(&args.output)?;
    if output == root || output.parent().is_none() {
        return Err("Refusing to replace the package root".to_string());
    }
    if output.exists() {
        if output != root.join(".native-packages")
            || fs::symlink_metadata(&output)
                .map_err(|error| error.to_string())?
                .file_type()
                .is_symlink()
        {
            return Err(format!(
                "Refusing to replace existing staging directory: {}",
                output.display()
            ));
        }
        fs::remove_dir_all(&output).map_err(|error| error.to_string())?;
    }
    fs::create_dir_all(&output).map_err(|error| error.to_string())?;
    let package_json: Value = serde_json::from_str(
        &fs::read_to_string(root.join("package.json")).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    if package_json["version"].as_str() != Some(env!("HARNESS_ALCHEMIST_VERSION")) {
        return Err("Compiled binary version does not match package.json".to_string());
    }

    let mut binaries = BTreeMap::<&str, PathBuf>::new();
    let has_artifacts = args.artifacts.is_some();
    if let Some(artifacts) = &args.artifacts {
        let artifacts = path_utils::absolute_normalized(artifacts)?;
        for target in TARGETS {
            let candidate = artifacts.join(target.triple).join(target.executable);
            if candidate.is_file() {
                binaries.insert(target.triple, candidate);
            }
        }
    }
    let host_binary = args.host_binary.or_else(|| {
        if !has_artifacts {
            let executable = if args.target.contains("windows") {
                "harness-alchemist.exe"
            } else {
                "harness-alchemist"
            };
            Some(root.join("target/release").join(executable))
        } else {
            None
        }
    });
    if let Some(binary) = host_binary {
        let binary = path_utils::absolute_normalized(&binary)?;
        if !binary.is_file() {
            return Err(format!("Host binary does not exist: {}", binary.display()));
        }
        let target = distribution_target(&args.target);
        binaries.insert(
            TARGETS
                .iter()
                .find(|candidate| candidate.triple == target)
                .ok_or_else(|| format!("Unsupported target: {}", args.target))?
                .triple,
            binary,
        );
    }
    if args.require_all && binaries.len() != TARGETS.len() {
        let missing = TARGETS
            .iter()
            .filter(|target| !binaries.contains_key(target.triple))
            .map(|target| target.triple)
            .collect::<Vec<_>>();
        return Err(format!("Missing native artifacts: {}", missing.join(", ")));
    }
    if binaries.is_empty() {
        return Err("No supported native artifacts were found".to_string());
    }
    for target in TARGETS {
        if let Some(binary) = binaries.get(target.triple) {
            stage_platform(&root, &output, &package_json, target, binary)?;
        }
    }
    stage_main(&root, &output, &package_json)?;
    println!(
        "Staged {} native package{} and the main npm package at {}",
        binaries.len(),
        if binaries.len() == 1 { "" } else { "s" },
        output.display()
    );
    Ok(())
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
    match stage(args) {
        Ok(()) => 0,
        Err(error) => {
            eprintln!("{error}");
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_target_has_a_unique_package() {
        let packages = TARGETS
            .iter()
            .map(|target| target.package)
            .collect::<std::collections::HashSet<_>>();
        assert_eq!(packages.len(), TARGETS.len());
    }

    #[test]
    fn stages_every_platform_manifest_and_binary() {
        let root = path_utils::package_root().expect("package root");
        let package_json: Value = serde_json::from_str(
            &fs::read_to_string(root.join("package.json")).expect("package.json"),
        )
        .expect("valid package.json");
        let temporary = tempfile::tempdir().expect("temporary directory");
        let artifacts = temporary.path().join("artifacts");
        let output = temporary.path().join("packages");

        for target in TARGETS {
            let binary = artifacts.join(target.triple).join(target.executable);
            fs::create_dir_all(binary.parent().expect("binary parent")).expect("artifact path");
            fs::write(&binary, target.triple).expect("artifact binary");
            stage_platform(&root, &output, &package_json, target, &binary)
                .expect("stage platform package");

            let directory = package_directory(&output, target.package);
            let manifest: Value = serde_json::from_str(
                &fs::read_to_string(directory.join("package.json")).expect("platform manifest"),
            )
            .expect("valid platform manifest");
            assert_eq!(manifest["name"], target.package);
            assert_eq!(manifest["os"][0], target.os);
            assert_eq!(manifest["cpu"][0], target.cpu);
            assert!(directory.join(target.executable).is_file());

            #[cfg(unix)]
            if target.os != "win32" {
                use std::os::unix::fs::PermissionsExt;
                let mode = fs::metadata(directory.join(target.executable))
                    .expect("staged binary metadata")
                    .permissions()
                    .mode();
                assert_ne!(mode & 0o111, 0);
            }
        }

        stage_main(&root, &output, &package_json).expect("stage main package");
        let main_manifest: Value = serde_json::from_str(
            &fs::read_to_string(output.join("main/package.json")).expect("main manifest"),
        )
        .expect("valid main manifest");
        assert_eq!(
            main_manifest["optionalDependencies"]
                .as_object()
                .expect("optional dependencies")
                .len(),
            TARGETS.len()
        );
    }

    #[test]
    fn refuses_to_replace_arbitrary_staging_directories() {
        let temporary = tempfile::tempdir().expect("temporary directory");
        let output = temporary.path().join("occupied");
        fs::create_dir(&output).expect("staging directory");
        fs::write(output.join("keep.txt"), "keep\n").expect("sentinel");

        let error = stage(Args {
            output: output.clone(),
            artifacts: None,
            host_binary: None,
            target: env!("HARNESS_ALCHEMIST_TARGET").to_string(),
            require_all: false,
        })
        .expect_err("refuse occupied directory");
        assert!(error.contains("Refusing to replace existing staging directory"));
        assert_eq!(
            fs::read_to_string(output.join("keep.txt")).expect("sentinel remains"),
            "keep\n"
        );
    }
}
