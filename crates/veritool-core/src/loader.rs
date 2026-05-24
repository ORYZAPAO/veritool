use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Context;
use sv_parser::{parse_sv, Define, DefineText};

use crate::design::Design;

pub struct FileList {
    pub files: Vec<PathBuf>,
    pub include_dirs: Vec<PathBuf>,
    pub defines: Vec<(String, Option<String>)>,
}

pub fn parse_filelist(filelist_path: &PathBuf) -> anyhow::Result<FileList> {
    let content = std::fs::read_to_string(filelist_path)
        .with_context(|| format!("Failed to read filelist: {}", filelist_path.display()))?;

    let mut files = Vec::new();
    let mut include_dirs = Vec::new();
    let mut defines = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("//") || line.starts_with('#') {
            continue;
        }

        if line.starts_with("-f") || line.starts_with("-F") {
            let subpath_str = line[2..].trim();
            let subpath = resolve_path(filelist_path, subpath_str);
            let sub = parse_filelist(&subpath)?;
            files.extend(sub.files);
            include_dirs.extend(sub.include_dirs);
            defines.extend(sub.defines);
        } else if line.starts_with("+incdir+") {
            let dir = &line[8..];
            include_dirs.push(resolve_path(filelist_path, dir));
        } else if line.starts_with("-D") {
            let def = line[2..].trim();
            if let Some(eq_pos) = def.find('=') {
                defines.push((def[..eq_pos].to_string(), Some(def[eq_pos + 1..].to_string())));
            } else {
                defines.push((def.to_string(), None));
            }
        } else if line.ends_with(".v") || line.ends_with(".sv") {
            files.push(resolve_path(filelist_path, line));
        }
    }

    Ok(FileList { files, include_dirs, defines })
}

fn resolve_path(base: &PathBuf, rel: &str) -> PathBuf {
    if rel.starts_with('/') {
        PathBuf::from(rel)
    } else {
        base.parent().unwrap_or_else(|| std::path::Path::new(".")).join(rel)
    }
}

pub fn parse_sv_files(
    file_paths: &[PathBuf],
    include_dirs: &[PathBuf],
    defines: &[(String, Option<String>)],
) -> anyhow::Result<Design> {
    let mut all_defines: HashMap<String, Option<Define>> = HashMap::new();
    for (name, value) in defines {
        let text = value.as_ref().map(|v| DefineText::new(v.clone(), None));
        let define = Define::new(name.clone(), vec![], text);
        all_defines.insert(name.clone(), Some(define));
    }

    let mut design = Design {
        modules: indexmap::IndexMap::new(),
        files: file_paths.to_vec(),
    };

    for path in file_paths {
        match parse_sv(path, &all_defines, include_dirs, false, false) {
            Ok((syntax_tree, new_defines)) => {
                crate::visit::visit_syntax_tree(&syntax_tree, path, &mut design);
                // Accumulate defines across files (for `define propagation)
                all_defines.extend(new_defines);
            }
            Err(e) => {
                eprintln!("Warning: failed to parse {}: {}", path.display(), e);
            }
        }
    }

    Ok(design)
}
