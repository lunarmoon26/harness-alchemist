use std::path::{Component, Path, PathBuf};

pub fn absolute_normalized(path: &Path) -> Result<PathBuf, String> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|error| error.to_string())?
            .join(path)
    };
    let mut normalized = PathBuf::new();
    for component in absolute.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    Ok(normalized)
}

pub fn is_contained(root: &Path, candidate: &Path) -> bool {
    candidate == root || candidate.starts_with(root)
}

pub fn find_project_root(start: &Path) -> Option<PathBuf> {
    let mut current = absolute_normalized(start).ok()?;
    let mut conventional = None;
    loop {
        if current.join("alchemy.json").exists() {
            return Some(current);
        }
        if conventional.is_none()
            && current.join("package.json").exists()
            && current.join(".claude-plugin").exists()
            && current.join(".codex-plugin").exists()
        {
            conventional = Some(current.clone());
        }
        if !current.pop() {
            return conventional;
        }
    }
}

pub fn package_root() -> Option<PathBuf> {
    if let Some(root) = std::env::var_os("HARNESS_ALCHEMIST_PACKAGE_ROOT") {
        return absolute_normalized(Path::new(&root)).ok();
    }
    let executable = std::env::current_exe().ok()?;
    find_project_root(executable.parent()?)
}
