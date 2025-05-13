//! 测试模板依赖管理器
//!
//! 这个模块负责分析和管理测试模板之间的依赖关系，确保它们以正确的顺序执行。
//! 当一个模板引用了另一个模板的变量时，它必须在被引用模板执行完成后才能执行。

use std::collections::{HashMap, HashSet, VecDeque};
use anyhow::{Result};
use log::{debug, warn};
use regex::Regex;

use crate::template::step::{ExecutionStep, GlobalStepId, StepType};

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
                    let var_name_in_command = var_match.as_str(); // Variable name as used in the command

                    for (potential_source_global_id, source_step) in &self.nodes {
                        if source_step.template_id == step.template_id { // Implicit dependencies are within the same template
                            let defines_var = if let Some(parsed_source) = &source_step.original_parsed_step {
                                parsed_source.extractions.iter().any(|ext| {
                                    // ext.variable is the actual name of the variable defined by source_step
                                    // var_name_in_command is the name used in the current step's command
                                    
                                    // Case 1: Command uses a simple variable name (e.g., ${my_var})
                                    (ext.variable == var_name_in_command) ||
                                    // Case 2: Command uses a local_id qualified variable name (e.g., ${step_local_id::my_var})
                                    (format!("{}::{}", source_step.local_id, ext.variable) == var_name_in_command) ||
                                    // Case 3: Command uses a fully qualified variable name (e.g., ${template_id::step_local_id::my_var})
                                    (format!("{}::{}::{}", source_step.template_id, source_step.local_id, ext.variable) == var_name_in_command)
                                })
                            } else { false };

                            if defines_var {
                                // Ensure a step does not depend on itself implicitly.
                                if current_step_id != potential_source_global_id {
                                    debug!(
                                        "Found implicit dependency: {} depends on {} due to variable {}",
                                        current_step_id, potential_source_global_id, var_name_in_command
                                    );
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

    /// 也许有必要，以后自动基于亲子关系设定执行依赖
    /// 比如，代码块依赖于它的父标题
    #[allow(dead_code)]
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

    #[test_log::test]
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
        for step in &order {
            debug!("Execution order: {}", step);
        }
        
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