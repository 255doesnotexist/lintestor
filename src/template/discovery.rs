//! 测试模板发现模块
//!
//! 这个模块负责扫描工作目录下的测试模板文件，并提供过滤功能

use anyhow::Result;
use log::{debug, info, warn};
use std::fs;
use std::path::{Path, PathBuf};

use crate::template::TestTemplate;

/// 模板过滤条件
#[derive(Debug, Clone, Default)]
pub struct TemplateFilter {
    /// 目标名称（可选）
    pub target: Option<String>,
    /// 单元名称（可选）
    pub unit: Option<String>,
    /// 标签列表（可选）
    pub tags: Vec<String>,
}

/// 在指定目录下发现测试模板
pub fn discover_templates<P: AsRef<Path>>(dir: P, recursive: bool) -> Result<Vec<PathBuf>> {
    let dir = dir.as_ref();
    let mut templates = Vec::new();

    if !dir.exists() {
        debug!("Directory does not exist: {}", dir.display()); // 目录不存在: {}
        return Ok(templates);
    }

    if !dir.is_dir() {
        debug!("Path is not a directory: {}", dir.display()); // 路径不是目录: {}
        return Ok(templates);
    }

    walk_directory(dir, &mut templates, recursive)?;

    info!(
        "Found {} test templates under {}",  // 在 {} 下发现了 {} 个测试模板
        templates.len(),
        dir.display()
    );

    Ok(templates)
}

/// 递归遍历目录，查找测试模板
fn walk_directory(dir: &Path, templates: &mut Vec<PathBuf>, recursive: bool) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() && recursive {
            walk_directory(&path, templates, recursive)?;
        } else if path.is_file() {
            // 检查是否是.test.md文件
            if let Some(ext) = path.extension() {
                if let Some(file_stem) = path.file_stem() {
                    let file_name = file_stem.to_string_lossy();
                    if ext == "md"
                        && (path.to_string_lossy().ends_with(".test.md")
                            || file_name.ends_with(".test"))
                    {
                        templates.push(path.clone());
                    }
                }
            }
        }
    }

    Ok(())
}

/// 根据过滤条件过滤测试模板
pub fn filter_templates(
    template_paths: Vec<PathBuf>,
    filter: &TemplateFilter,
) -> Result<Vec<TestTemplate>> {
    let mut filtered_templates = Vec::new();

    for path in &template_paths {
        match TestTemplate::from_file(path) {
            Ok(template) => {
                // 应用过滤条件
                if matches_filter(&template, filter) {
                    filtered_templates.push(template);
                }
            }
            Err(e) => {
                warn!("Failed to parse template {}: {}", path.display(), e); // 解析模板 {} 失败: {}
                continue;
            }
        }
    }

    info!("Remaining {} test templates after filtering", filtered_templates.len()); // 过滤后剩余 {} 个测试模板

    Ok(filtered_templates)
}

/// 检查模板是否匹配过滤条件
fn matches_filter(template: &TestTemplate, filter: &TemplateFilter) -> bool {
    // 检查目标名称
    if let Some(ref target) = filter.target {
        // 直接通过 get_name() 获取目标名称进行比较
        if template.metadata.target_config.get_name() != *target {
            return false;
        }
    }

    // 检查单元名称
    if let Some(ref unit) = filter.unit {
        if template.metadata.unit_name != *unit {
            return false;
        }
    }

    // 检查标签
    if !filter.tags.is_empty() {
        let mut has_matching_tag = false;

        for tag in &filter.tags {
            if template.metadata.tags.contains(tag) {
                has_matching_tag = true;
                break;
            }
        }

        if !has_matching_tag {
            return false;
        }
    }

    true
}
