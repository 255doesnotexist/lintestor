//! 测试模板依赖管理器
//!
//! 这个模块负责分析和管理测试模板之间的依赖关系，确保它们以正确的顺序执行。
//! 当一个模板引用了另一个模板的变量时，它必须在被引用模板执行完成后才能执行。

use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use anyhow::{Result, Context, bail, anyhow};
use log::{debug, info, warn};

use crate::template::{TestTemplate, TemplateReference};

/// 模板依赖管理器
pub struct TemplateDependencyManager {
    /// 所有模板的映射 (路径 -> 模板)
    templates: HashMap<PathBuf, TestTemplate>,
    /// 依赖图 (依赖方 -> 被依赖方列表)
    dependency_graph: HashMap<PathBuf, Vec<PathBuf>>,
    /// 反向依赖图 (被依赖方 -> 依赖方列表)
    reverse_dependency_graph: HashMap<PathBuf, Vec<PathBuf>>,
    /// 排序后的执行顺序
    execution_order: Vec<PathBuf>,
    /// 工作目录
    work_dir: PathBuf,
}

impl TemplateDependencyManager {
    /// 创建新的模板依赖管理器
    pub fn new(work_dir: PathBuf) -> Self {
        Self {
            templates: HashMap::new(),
            dependency_graph: HashMap::new(),
            reverse_dependency_graph: HashMap::new(),
            execution_order: Vec::new(),
            work_dir,
        }
    }

    /// 添加模板
    pub fn add_template(&mut self, template: TestTemplate) -> Result<()> {
        let template_path = template.file_path.clone();
        
        // 确保路径是标准化的
        let template_path = self.normalize_path(&template_path)?;
        
        // 添加到模板映射
        self.templates.insert(template_path.clone(), template);
        
        // 初始化依赖图中该模板的条目
        self.dependency_graph.entry(template_path.clone()).or_insert_with(Vec::new);
        self.reverse_dependency_graph.entry(template_path).or_insert_with(Vec::new);
        
        Ok(())
    }

    /// 添加多个模板
    pub fn add_templates<I>(&mut self, templates: I) -> Result<()> 
    where
        I: IntoIterator<Item = TestTemplate>
    {
        for template in templates {
            self.add_template(template)?;
        }
        Ok(())
    }

    /// 构建依赖图
    pub fn build_dependency_graph(&mut self) -> Result<()> {
        info!("开始构建模板依赖图");
        
        // 首先清空现有依赖图
        self.dependency_graph.clear();
        self.reverse_dependency_graph.clear();
        
        // 遍历所有模板，初始化依赖图和反向依赖图
        for path in self.templates.keys() {
            self.dependency_graph.insert(path.clone(), Vec::new());
            self.reverse_dependency_graph.insert(path.clone(), Vec::new());
        }
        
        // 遍历所有模板，处理其引用
        for (template_path, template) in &self.templates {
            debug!("处理模板: {}", template_path.display());
            
            // 处理每个外部引用
            for reference in &template.metadata.references {
                debug!("  处理引用: {}, 命名空间: {}", reference.template_path, reference.namespace);
                
                // 解析被引用模板的完整路径
                let referenced_path = self.resolve_template_path(&reference.template_path)?;
                debug!("  解析到完整路径: {}", referenced_path.display());
                
                // 检查被引用的模板是否存在
                if !self.templates.contains_key(&referenced_path) {
                    warn!("模板 {} 引用了未知模板: {}", template_path.display(), referenced_path.display());
                    bail!("引用了未知模板: {}", referenced_path.display());
                }
                
                // 更新依赖图
                self.dependency_graph.get_mut(template_path).unwrap().push(referenced_path.clone());
                
                // 更新反向依赖图
                self.reverse_dependency_graph.get_mut(&referenced_path).unwrap().push(template_path.clone());
            }
        }
        
        // 检查是否有循环依赖
        self.check_circular_dependencies()?;
        
        // 使用拓扑排序生成执行顺序
        self.topological_sort()?;
        
        info!("模板依赖图构建完成，共有 {} 个执行单元", self.execution_order.len());
        
        Ok(())
    }

    /// 检查是否有循环依赖
    fn check_circular_dependencies(&self) -> Result<()> {
        debug!("检查模板间循环依赖");
        
        // 使用深度优先搜索检测循环
        let mut visited = HashSet::new();
        let mut path = HashSet::new();
        
        for template_path in self.templates.keys() {
            if !visited.contains(template_path) {
                if self.has_cycle(template_path, &mut visited, &mut path)? {
                    bail!("检测到循环依赖！");
                }
            }
        }
        
        Ok(())
    }

    /// 深度优先搜索检测循环
    fn has_cycle(
        &self, 
        current: &PathBuf, 
        visited: &mut HashSet<PathBuf>, 
        path: &mut HashSet<PathBuf>
    ) -> Result<bool> {
        // 标记当前节点为已访问
        visited.insert(current.clone());
        // 添加到当前路径
        path.insert(current.clone());
        
        // 检查所有依赖
        if let Some(dependencies) = self.dependency_graph.get(current) {
            for dependency in dependencies {
                // 如果依赖已在当前路径中，说明有循环
                if path.contains(dependency) {
                    // 找出循环路径用于输出
                    let mut cycle = Vec::new();
                    for p in path.iter() {
                        cycle.push(p.to_string_lossy().to_string());
                        if p == dependency {
                            break;
                        }
                    }
                    cycle.push(dependency.to_string_lossy().to_string());
                    
                    warn!("检测到循环依赖: {}", cycle.join(" -> "));
                    return Ok(true);
                }
                
                // 递归检查
                if !visited.contains(dependency) {
                    if self.has_cycle(dependency, visited, path)? {
                        return Ok(true);
                    }
                }
            }
        }
        
        // 回溯，从当前路径中移除
        path.remove(current);
        
        Ok(false)
    }

    /// 拓扑排序生成执行顺序
    fn topological_sort(&mut self) -> Result<()> {
        debug!("执行拓扑排序确定模板执行顺序");
        
        // 清空现有执行顺序
        self.execution_order.clear();
        
        // 计算每个节点的入度
        let mut in_degree = HashMap::new();
        for (template, _) in &self.templates {
            in_degree.insert(template.clone(), 0);
        }
        
        for (_, dependencies) in &self.dependency_graph {
            for dependency in dependencies {
                *in_degree.entry(dependency.clone()).or_insert(0) += 1;
            }
        }
        
        // 使用BFS进行拓扑排序
        let mut queue = VecDeque::new();
        
        // 将入度为0的节点加入队列
        for (template, degree) in &in_degree {
            if *degree == 0 {
                queue.push_back(template.clone());
            }
        }
        
        // 执行拓扑排序
        while let Some(template) = queue.pop_front() {
            self.execution_order.push(template.clone());
            
            if let Some(dependencies) = self.reverse_dependency_graph.get(&template) {
                for dependent in dependencies {
                    let entry = in_degree.get_mut(dependent).unwrap();
                    *entry -= 1;
                    
                    if *entry == 0 {
                        queue.push_back(dependent.clone());
                    }
                }
            }
        }
        
        // 如果执行顺序长度小于模板数量，说明有循环依赖
        if self.execution_order.len() != self.templates.len() {
            bail!("拓扑排序失败，可能存在循环依赖");
        }
        
        // 输出执行顺序
        debug!("拓扑排序结果:");
        for (i, template) in self.execution_order.iter().enumerate() {
            debug!("{}. {}", i + 1, template.display());
        }
        
        Ok(())
    }

    /// 标准化路径
    fn normalize_path(&self, path: &Path) -> Result<PathBuf> {
        // 如果是相对路径，转换为相对于工作目录的绝对路径
        if path.is_relative() {
            Ok(self.work_dir.join(path))
        } else {
            Ok(path.to_path_buf())
        }
    }

    /// 解析模板路径
    /// 将相对引用的模板路径解析为完整路径
    fn resolve_template_path(&self, template_path: &str) -> Result<PathBuf> {
        let mut path = PathBuf::new();
        
        // 添加默认目录
        path.push(&self.work_dir);
        path.push("tests");
        
        // 处理相对路径，如"unit/target.test.md"
        path.push(template_path);
        
        // 如果没有扩展名，添加默认扩展名
        if path.extension().is_none() {
            path.set_extension("test.md");
        }
        
        Ok(path)
    }

    /// 获取执行顺序
    pub fn get_execution_order(&self) -> &[PathBuf] {
        &self.execution_order
    }

    /// 获取模板
    pub fn get_template(&self, path: &Path) -> Option<&TestTemplate> {
        self.templates.get(path)
    }

    /// 获取模板的所有直接依赖
    pub fn get_dependencies(&self, path: &Path) -> Vec<PathBuf> {
        self.dependency_graph.get(path).cloned().unwrap_or_default()
    }

    /// 获取依赖于指定模板的所有模板
    pub fn get_dependents(&self, path: &Path) -> Vec<PathBuf> {
        self.reverse_dependency_graph.get(path).cloned().unwrap_or_default()
    }

    /// 将测试模板的路径转换为预期的报告文件路径
    pub fn get_report_path(&self, template_path: &Path, target_name: &str) -> PathBuf {
        let mut report_path = self.work_dir.clone();
        report_path.push("reports");
        
        // 获取模板文件名（不含扩展名）
        let file_stem = template_path.file_stem()
            .and_then(|os_str| os_str.to_str())
            .unwrap_or("unknown");
        
        // 根据命名约定，报告文件名是<template_name>_<target_name>.report.md
        let report_name = format!("{}_{}.report.md", file_stem, target_name);
        report_path.push(report_name);
        
        report_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::{TestTemplate, TemplateMetadata, TemplateReference};
    use std::path::PathBuf;
    
    fn create_test_template(
        path: &str, 
        references: Vec<TemplateReference>
    ) -> TestTemplate {
        // 创建一个简单的测试模板，只包含必要的字段
        let metadata = TemplateMetadata {
            title: format!("Test Template {}", path),
            target_config: PathBuf::from("targets/local/config.toml"),
            unit_name: "test_unit".to_string(),
            unit_version_command: None,
            tags: Vec::new(),
            references,
            custom: HashMap::new(),
        };
        
        TestTemplate {
            metadata,
            steps: Vec::new(),
            file_path: PathBuf::from(path),
            raw_content: String::new(),
        }
    }
    
    #[test]
    fn test_dependency_graph_simple() {
        // 创建一个简单的依赖图：A -> B -> C
        let work_dir = PathBuf::from("/tmp");
        let mut manager = TemplateDependencyManager::new(work_dir);
        
        let template_c = create_test_template("tests/C.test.md", vec![]);
        let template_b = create_test_template("tests/B.test.md", vec![
            TemplateReference {
                template_path: "C.test.md".to_string(),
                namespace: "C".to_string(),
            }
        ]);
        let template_a = create_test_template("tests/A.test.md", vec![
            TemplateReference {
                template_path: "B.test.md".to_string(),
                namespace: "B".to_string(),
            }
        ]);
        
        // 添加模板（注意添加顺序不影响结果）
        manager.add_template(template_a).unwrap();
        manager.add_template(template_b).unwrap();
        manager.add_template(template_c).unwrap();
        
        // 构建依赖图
        manager.build_dependency_graph().unwrap();
        
        // 验证执行顺序
        let execution_order = manager.get_execution_order();
        assert_eq!(execution_order.len(), 3);
        
        // 执行顺序应该是 C -> B -> A
        assert_eq!(execution_order[0].file_name().unwrap(), "C.test.md");
        assert_eq!(execution_order[1].file_name().unwrap(), "B.test.md");
        assert_eq!(execution_order[2].file_name().unwrap(), "A.test.md");
    }
    
    #[test]
    fn test_circular_dependency_detection() {
        // 创建一个循环依赖：A -> B -> C -> A
        let work_dir = PathBuf::from("/tmp");
        let mut manager = TemplateDependencyManager::new(work_dir);
        
        let template_a = create_test_template("tests/A.test.md", vec![
            TemplateReference {
                template_path: "B.test.md".to_string(),
                namespace: "B".to_string(),
            }
        ]);
        let template_b = create_test_template("tests/B.test.md", vec![
            TemplateReference {
                template_path: "C.test.md".to_string(),
                namespace: "C".to_string(),
            }
        ]);
        let template_c = create_test_template("tests/C.test.md", vec![
            TemplateReference {
                template_path: "A.test.md".to_string(),
                namespace: "A".to_string(),
            }
        ]);
        
        // 添加模板
        manager.add_template(template_a).unwrap();
        manager.add_template(template_b).unwrap();
        manager.add_template(template_c).unwrap();
        
        // 构建依赖图应该失败，因为有循环依赖
        let result = manager.build_dependency_graph();
        assert!(result.is_err());
    }
}