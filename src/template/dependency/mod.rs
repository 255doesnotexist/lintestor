//! 测试模板依赖管理器
//!
//! 这个模块负责分析和管理测试模板之间的依赖关系，确保它们以正确的顺序执行。
//! 当一个模板引用了另一个模板的变量时，它必须在被引用模板执行完成后才能执行。

use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use anyhow::{Result, bail};
use log::{debug, info, warn};
use regex::Regex;

use crate::template::{TestTemplate, DataExtraction};
use crate::template::step::{ExecutionStep, GlobalStepId, StepType};

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
        
        // 首先收集需要加载的模板
        let mut templates_to_load = Vec::new();
        
        // 遍历所有模板，收集需要加载的引用模板
        for (template_path, template) in &self.templates {
            debug!("处理模板: {}", template_path.display());
            
            // 处理每个外部引用
            for reference in &template.metadata.references {
                debug!("  处理引用: {}, 命名空间: {}", reference.template_path, reference.namespace);
                
                // 解析被引用模板的完整路径
                let referenced_path = self.resolve_template_path(&reference.template_path)?;
                debug!("  解析到完整路径: {}", referenced_path.display());
                
                // 检查被引用的模板是否存在于模板集合中
                if !self.templates.contains_key(&referenced_path) {
                    // 如果文件确实存在但我们没有加载它，添加到待加载列表
                    if referenced_path.exists() {
                        debug!("  添加到待加载列表: {}", referenced_path.display());
                        templates_to_load.push(referenced_path.clone());
                    } else {
                        warn!("模板 {} 引用了不存在的模板: {}", template_path.display(), referenced_path.display());
                        bail!("引用了不存在的模板: {}", referenced_path.display());
                    }
                }
            }
        }
        
        // 加载所有收集到的模板
        for path in templates_to_load {
            debug!("加载引用的模板文件: {}", path.display());
            match TestTemplate::from_file(&path) {
                Ok(loaded_template) => {
                    info!("成功加载引用的模板: {}", path.display());
                    self.templates.insert(path.clone(), loaded_template);
                    self.dependency_graph.insert(path.clone(), Vec::new());
                    self.reverse_dependency_graph.insert(path.clone(), Vec::new());
                },
                Err(e) => {
                    warn!("无法加载引用的模板: {}, 错误: {}", path.display(), e);
                    bail!("无法加载引用的模板: {}, 错误: {}", path.display(), e);
                }
            }
        }
        
        // 再次遍历所有模板，处理依赖关系
        for (template_path, template) in &self.templates {
            // 处理每个外部引用
            for reference in &template.metadata.references {
                // 解析被引用模板的W完整路径
                let referenced_path = self.resolve_template_path(&reference.template_path)?;
                
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

    /// 解析模板路径 (或直接在工作目录递归查找)
    fn resolve_template_path(&self, template_path: &str) -> Result<PathBuf> {
        // 如果是绝对路径，直接返回
        let path = PathBuf::from(template_path);
        if path.is_absolute() {
            if path.exists() {
                return Ok(path);
            } else {
                bail!("指定的绝对路径模板不存在: {}", path.display());
            }
        }
        
        // 首先尝试相对于工作目录直接解析
        let direct_path = self.work_dir.join(template_path);
        if direct_path.exists() {
            return Ok(direct_path);
        }
        
        // 如果没有扩展名，添加默认扩展名后再试一次
        let mut with_extension = direct_path.clone();
        if with_extension.extension().is_none() {
            with_extension.set_extension("test.md");
            if with_extension.exists() {
                return Ok(with_extension);
            }
        }
        
        // 在现有模板集合中查找匹配的文件名
        let file_name = Path::new(template_path).file_name().unwrap_or_default();
        for (path, _) in &self.templates {
            if path.file_name().unwrap_or_default() == file_name {
                return Ok(path.clone());
            }
        }
        
        // 递归查找报告目录下的所有测试模板
        info!("在报告目录中递归查找模板: {}", template_path);
        let reports_dir = self.work_dir.join("reports");
        
        if reports_dir.exists() {
            // 先尝试完整匹配文件路径
            let mut matches = Vec::new();
            self.find_template_files(&reports_dir, Some(template_path), &mut matches)?;
            
            if !matches.is_empty() {
                // 找到精确匹配
                debug!("找到精确匹配的模板: {}", matches[0].display());
                return Ok(matches[0].clone());
            }
            
            // 如果没有找到精确匹配，尝试匹配文件名
            let mut name_matches = Vec::new();
            self.find_template_files(&reports_dir, None, &mut name_matches)?;
            
            for path in name_matches {
                if let Some(file_name_from_path) = path.file_name() {
                    if file_name_from_path == file_name {
                        debug!("通过文件名匹配到模板: {}", path.display());
                        return Ok(path);
                    }
                }
            }
        }
        
        // 尝试在 tests 目录下查找
        let tests_dir = self.work_dir.join("tests");
        if tests_dir.exists() {
            let mut matches = Vec::new();
            self.find_template_files(&tests_dir, Some(template_path), &mut matches)?;
            
            if !matches.is_empty() {
                // 找到精确匹配
                debug!("在 tests 目录中找到模板: {}", matches[0].display());
                return Ok(matches[0].clone());
            }
            
            // 如果没有找到精确匹配，尝试匹配文件名
            let mut name_matches = Vec::new();
            self.find_template_files(&tests_dir, None, &mut name_matches)?;
            
            for path in name_matches {
                if let Some(file_name_from_path) = path.file_name() {
                    if file_name_from_path == file_name {
                        debug!("在 tests 目录中通过文件名匹配到模板: {}", path.display());
                        return Ok(path);
                    }
                }
            }
        }
        
        // 如果所有查找都失败，返回错误
        bail!("无法找到模板: {}", template_path)
    }

    /// 递归查找目录中的模板文件
    fn find_template_files(&self, dir: &Path, partial_path: Option<&str>, matches: &mut Vec<PathBuf>) -> Result<()> {
        if !dir.is_dir() {
            return Ok(());
        }
        
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                // 递归查找子目录
                self.find_template_files(&path, partial_path, matches)?;
            } else {
                // 检查是否是 .test.md 文件
                if let Some(ext) = path.extension() {
                    if ext == "md" && path.to_string_lossy().contains(".test.") {
                        if let Some(partial) = partial_path {
                            // 如果指定了部分路径，检查文件路径是否包含该部分
                            let path_str = path.to_string_lossy().to_lowercase();
                            let partial_lower = partial.to_lowercase();
                            
                            if path_str.contains(&partial_lower) {
                                matches.push(path.clone());
                            }
                        } else {
                            // 不检查部分路径，收集所有 .test.md 文件
                            matches.push(path.clone());
                        }
                    }
                }
            }
        }
        
        Ok(())
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
        self.dependency_graph.get(path).cloned().unwrap_or_else(Vec::new)
    }

    /// 获取依赖于指定模板的所有模板
    pub fn get_dependents(&self, path: &Path) -> Vec<PathBuf> {
        self.reverse_dependency_graph.get(path).cloned().unwrap_or_else(Vec::new)
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

/// Manages dependencies between individual execution steps.
#[derive(Debug)]
pub struct StepDependencyManager {
    /// All known execution steps, mapped by their global ID.
    nodes: HashMap<GlobalStepId, ExecutionStep>,
    /// Graph: step A -> {steps that A depends on}.
    graph: HashMap<GlobalStepId, NodeData>,
}

#[derive(Debug, Default)]
pub struct NodeData {
    dependencies: HashSet<GlobalStepId>,
    dependents: HashSet<GlobalStepId>,
}

impl StepDependencyManager {
    pub fn new() -> Self {
        StepDependencyManager {
            nodes: HashMap::new(),
            graph: HashMap::new(),
        }
    }

    pub fn add_steps(&mut self, steps: Vec<ExecutionStep>) {
        for step in steps {
            self.graph.entry(step.id.clone()).or_default(); // Ensure node exists in graph
            for dep_id in &step.dependencies { // dependencies is HashSet<GlobalStepId>
                self.graph.entry(step.id.clone()).or_default().dependencies.insert(dep_id.clone());
                self.graph.entry(dep_id.clone()).or_default().dependents.insert(step.id.clone());
            }
            self.extract_dependencies_from_step(&step); // Implicit dependencies
            self.nodes.insert(step.id.clone(), step);
        }
    }

    fn extract_dependencies_from_step(&mut self, step: &ExecutionStep) {
        let var_regex = match Regex::new(r"\$\{\s*([a-zA-Z0-9_]+(?:[:]{2}[a-zA-Z0-9_]+)?)\s*\}") {
            Ok(r) => r,
            Err(e) => {
                warn!("Failed to compile regex for variable extraction: {}", e);
                return;
            }
        };
        let current_step_id = &step.id;

        let command_to_check = match &step.step_type {
            StepType::CodeBlock { command, .. } => Some(command.as_str()),
            _ => None,
        };

        if let Some(command) = command_to_check {
            for caps in var_regex.captures_iter(command) {
                if let Some(var_match) = caps.get(1) {
                    let var_name = var_match.as_str();
                    if let Some(potential_source_local_id_str) = var_name.split("::").next() {
                        for (potential_source_global_id, source_step) in &self.nodes {
                            if source_step.local_id == potential_source_local_id_str && source_step.template_id == step.template_id {
                                let defines_var = if let Some(parsed_source) = &source_step.original_parsed_step {
                                    parsed_source.extractions.iter().any(|ext| {
                                        ext.variable == var_name || 
                                        format!("{}::{}", source_step.local_id, ext.variable) == var_name || 
                                        format!("{}::{}::{}", source_step.template_id, source_step.local_id, ext.variable) == var_name 
                                    })
                                } else { false };

                                if defines_var {
                                    debug!("Found implicit dependency: {} depends on {} due to variable {}", current_step_id, potential_source_global_id, var_name);
                                    self.graph.entry(current_step_id.clone()).or_default().dependencies.insert(potential_source_global_id.clone());
                                    self.graph.entry(potential_source_global_id.clone()).or_default().dependents.insert(current_step_id.clone());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn build_graph(&mut self) {
        for step_id in self.nodes.keys() {
            self.graph.entry(step_id.clone()).or_default();
        }
    }

    pub fn get_execution_order(&self) -> Result<Vec<GlobalStepId>, String> {
        self.topological_sort()
    }

    fn topological_sort(&self) -> Result<Vec<GlobalStepId>, String> {
        let mut in_degree: HashMap<GlobalStepId, usize> = HashMap::new();
        let mut queue = VecDeque::new();
        let mut sorted_order = Vec::new();

        for (node_id, node_data) in &self.graph {
            let current_in_degree = node_data.dependencies.len();
            in_degree.insert(node_id.clone(), current_in_degree);
            if current_in_degree == 0 {
                queue.push_back(node_id.clone());
            }
        }
        
        if self.nodes.is_empty() {
            return Ok(sorted_order);
        }
        
        if queue.is_empty() && !self.nodes.is_empty() && self.graph.values().any(|nd| !nd.dependencies.is_empty()) {
            let all_nodes_have_deps = self.graph.iter().all(|(id, data)| {
                data.dependencies.len() > 0 || self.graph.values().all(|other_data| !other_data.dependents.contains(id))
            });
            if all_nodes_have_deps && self.graph.len() > 1 {
                warn!("Topological sort: Queue is empty but nodes exist, possible cycle. Graph: {:?}", self.graph);
                return Err("Circular dependency detected or all nodes have dependencies.".to_string());
            }
        }

        while let Some(u_id) = queue.pop_front() {
            sorted_order.push(u_id.clone());

            if let Some(node_data) = self.graph.get(&u_id) {
                for v_id in &node_data.dependents {
                    if let Some(degree) = in_degree.get_mut(v_id) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(v_id.clone());
                        }
                    } else {
                        warn!("Topological sort: Dependent node {} not found in in_degree map.", v_id);
                    }
                }
            }
        }

        if sorted_order.len() != self.nodes.len() {
            let mut missing_nodes = Vec::new();
            let sorted_set: HashSet<_> = sorted_order.iter().collect();
            for node_id in self.nodes.keys() {
                if !sorted_set.contains(node_id) {
                    missing_nodes.push(node_id.clone());
                }
            }
            warn!(
                "Topological sort error: Order length {} does not match node count {}. Missing: {:?}. Graph: {:?}. In-degrees: {:?}",
                sorted_order.len(),
                self.nodes.len(),
                missing_nodes,
                self.graph,
                in_degree
            );
            Err(format!("Circular dependency detected or graph inconsistency. Processed {} nodes, expected {}. Missing: {:?}", sorted_order.len(), self.nodes.len(), missing_nodes))
        } else {
            Ok(sorted_order)
        }
    }

    pub fn get_step(&self, step_id: &GlobalStepId) -> Option<&ExecutionStep> {
        self.nodes.get(step_id)
    }

    pub fn identify_parent_headings(&self, step: &ExecutionStep) -> Vec<GlobalStepId> {
        let mut parents = Vec::new();
        let mut current_id_to_check = step.id.clone();

        loop {
            let current_node = self.nodes.get(&current_id_to_check);
            if current_node.is_none() { break; }

            let mut found_parent_heading = false;
            if let Some(node_data) = self.graph.get(&current_id_to_check) {
                for dep_id in &node_data.dependencies {
                    if let Some(dep_step) = self.nodes.get(dep_id) {
                        match &dep_step.step_type {
                            StepType::Heading { .. } => {
                                parents.push(dep_id.clone());
                                current_id_to_check = dep_id.clone();
                                found_parent_heading = true;
                                break; 
                            }
                            _ => {}
                        }
                    }
                }
            }
            if !found_parent_heading { break; }
        }
        parents.reverse();
        parents
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::{step::StepType, ParsedTestStep};

    fn create_test_execution_step(
        template_id: &str,
        local_id: &str,
        step_type: StepType,
        dependencies: Vec<GlobalStepId>,
        original_parsed: Option<ParsedTestStep>
    ) -> ExecutionStep {
        ExecutionStep {
            id: format!("{}::{}", template_id, local_id),
            template_id: template_id.to_string(),
            local_id: local_id.to_string(),
            step_type,
            dependencies: dependencies.into_iter().collect(),
            original_parsed_step: original_parsed,
        }
    }
    
    fn create_mock_parsed_step(id: &str, command: Option<&str>, extractions: Vec<DataExtraction>) -> ParsedTestStep {
        ParsedTestStep {
            id: id.to_string(),
            description: Some(format!("Parsed step {}", id)),
            command: command.map(String::from),
            depends_on: Vec::new(),
            assertions: Vec::new(),
            extractions,
            executable: command.is_some(),
            ref_command: None,
            raw_content: String::new(),
            active: None, // Added missing field
            timeout_ms: None, // Added missing field
        }
    }

    #[test]
    fn test_add_steps_and_get_order_simple_linear() {
        let mut manager = StepDependencyManager::new();
        let step1 = create_test_execution_step("t1", "s1", StepType::CodeBlock { lang: "bash".into(), command: "echo s1".into(), attributes: HashMap::new() }, vec![], None);
        let step2_id = format!("{}::s1", "t1");
        let step2 = create_test_execution_step("t1", "s2", StepType::CodeBlock { lang: "bash".into(), command: "echo s2".into(), attributes: HashMap::new() }, vec![step2_id], None);
        
        manager.add_steps(vec![step1.clone(), step2.clone()]);
        manager.build_graph();
        let order = manager.get_execution_order().unwrap();
        
        assert_eq!(order, vec![step1.id, step2.id]);
    }

    #[test]
    fn test_circular_dependency() {
        let mut manager = StepDependencyManager::new();
        let step1_id = format!("{}::s2", "t1");
        let step1 = create_test_execution_step("t1", "s1", StepType::CodeBlock { lang: "bash".into(), command: "echo s1".into(), attributes: HashMap::new() }, vec![step1_id], None);
        
        let step2_id = format!("{}::s1", "t1");
        let step2 = create_test_execution_step("t1", "s2", StepType::CodeBlock { lang: "bash".into(), command: "echo s2".into(), attributes: HashMap::new() }, vec![step2_id], None);

        manager.add_steps(vec![step1.clone(), step2.clone()]);
        manager.build_graph();
        let result = manager.get_execution_order();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_lowercase().contains("circular dependency"));
    }

    #[test]
    fn test_implicit_dependency_from_variable() {
        let mut manager = StepDependencyManager::new();

        let extraction_s1 = vec![DataExtraction { variable: "my_var".to_string(), regex: ".*".to_string() }];
        let parsed_s1 = create_mock_parsed_step("s1", Some("echo 'output_for_s1'"), extraction_s1);
        let step1 = create_test_execution_step(
            "t1", "s1", 
            StepType::CodeBlock { lang: "bash".into(), command: "echo 'output_for_s1'".into(), attributes: HashMap::new() }, 
            vec![], 
            Some(parsed_s1)
        );

        let step2 = create_test_execution_step(
            "t1", "s2", 
            StepType::CodeBlock { lang: "bash".into(), command: "echo ${my_var}".into(), attributes: HashMap::new() }, 
            vec![], 
            None
        );
        
        manager.add_steps(vec![step1.clone(), step2.clone()]);
        manager.build_graph();
        
        let order = manager.get_execution_order().unwrap_or_else(|e| panic!("Execution order failed: {}", e));
        
        assert_eq!(order.len(), 2);
        assert_eq!(order[0], step1.id);
        assert_eq!(order[1], step2.id);
        assert!(manager.graph.get(&step2.id).unwrap().dependencies.contains(&step1.id));
    }
    
    #[test]
    fn test_identify_parent_headings_simple_case() {
        let mut manager = StepDependencyManager::new();

        let h1 = create_test_execution_step("doc", "h1", StepType::Heading { level: 1, text: "H1".into(), attributes: HashMap::new() }, vec![], None);
        let h2_id = h1.id.clone();
        let h2 = create_test_execution_step("doc", "h2", StepType::Heading { level: 2, text: "H2".into(), attributes: HashMap::new() }, vec![h2_id], None);
        let code_id = h2.id.clone();
        let code = create_test_execution_step("doc", "code1", StepType::CodeBlock { lang: "sh".into(), command: "ls".into(), attributes: HashMap::new() }, vec![code_id], None);
        
        manager.add_steps(vec![h1.clone(), h2.clone(), code.clone()]);
        manager.build_graph();

        let parents = manager.identify_parent_headings(&code);
        assert_eq!(parents, vec![h1.id, h2.id]);
    }
}