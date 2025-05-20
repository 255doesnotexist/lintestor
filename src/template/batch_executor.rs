//! 批量模板执行器
//!
//! 这个模块负责按照依赖关系顺序执行多个测试模板，
//! 管理代码块级别的依赖并收集执行结果

use anyhow::{anyhow, Result};
use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::config::target_config::TargetConfig;
use crate::connection::ConnectionFactory;
use crate::template::dependency::StepDependencyManager;
use crate::template::executor::{check_assertion, extract_variable, ExecutionResult};
use crate::template::reporter::Reporter;
use crate::template::step::{GlobalStepId, StepType};
use crate::template::variable::VariableManager;
use crate::template::{BatchOptions, StepStatus, TestTemplate};
use crate::utils;

/// 批量测试执行器
pub struct BatchExecutor {
    variable_manager: VariableManager,
    step_dependency_manager: StepDependencyManager,
    executed_step_results: HashMap<GlobalStepId, crate::template::executor::StepResult>,
    templates: HashMap<String, Arc<TestTemplate>>,
    options: Option<BatchOptions>,
    report_dir: Option<PathBuf>,
}

impl BatchExecutor {
    pub fn new(variable_manager: VariableManager, options: Option<BatchOptions>) -> Self {
        let report_dir = options.as_ref().and_then(|o| o.report_directory.clone());
        Self {
            variable_manager,
            step_dependency_manager: StepDependencyManager::new(),
            executed_step_results: HashMap::new(),
            templates: HashMap::new(),
            options,
            report_dir,
        }
    }

    pub fn add_template(&mut self, template: Arc<TestTemplate>) -> Result<(), Box<dyn Error>> {
        let template_id = template.get_template_id();
        // 在这里注册一下模板
        if !self.variable_manager.template_id_exists(&template_id) {
            self.variable_manager
                .register_template(&template, Some(&template_id))?;
        }
        self.templates.insert(template.get_template_id(), template);
        Ok(())
    }

    pub fn execute(
        &mut self,
        template_id: &str,
    ) -> Result<ExecutionResult, Box<dyn std::error::Error>> {
        info!("Executing template: {template_id}");
        let start_time_total = Instant::now();

        let template_arc = match self.templates.get(template_id) {
            Some(t) => t.clone(),
            None => {
                return Err(anyhow!("Template {} not found in BatchExecutor", template_id).into())
            }
        };

        // Now you could use template_id::metadata.custom_field to read the custom field in the template metadata
        // This map append "metadata." at the front of the key
        let custom_fields_map: HashMap<String, String> = template_arc
            .metadata
            .custom
            .iter()
            .map(|(k, v)| (format!("metadata.{k}"), v.clone()))
            .collect::<HashMap<String, String>>();

        self.variable_manager.set_variables_from_map(&template_arc.get_template_id(), "GLOBAL", 
            &custom_fields_map)?;

        self.step_dependency_manager = StepDependencyManager::new();

        let execution_steps_from_template = template_arc.steps.clone();
        if execution_steps_from_template.is_empty() {
            warn!("Template {template_id} has no executable steps.");
            return Ok(ExecutionResult {
                template: template_arc.clone(),
                unit_name: template_arc.metadata.unit_name.clone(),
                target_name: template_arc
                    .metadata
                    .target_config
                    .get_name()
                    .to_string(),
                overall_status: StepStatus::Skipped,
                step_results: HashMap::new(),
                variables: self.variable_manager.get_all_variables().clone(),
                special_vars: HashMap::new(),
                report_path: None,
            });
        }

        self.step_dependency_manager
            .add_steps(execution_steps_from_template);
        self.step_dependency_manager.build_graph();

        let execution_order = match self.step_dependency_manager.get_execution_order() {
            Ok(order) => order,
            Err(e) => {
                error!(
                    "Failed to get execution order for template {template_id}: {e}"
                );
                return Ok(ExecutionResult {
                    template: template_arc.clone(),
                    unit_name: template_arc.metadata.unit_name.clone(),
                    target_name: template_arc
                        .metadata
                        .target_config
                        .get_name()
                        .to_string(),
                    overall_status: StepStatus::Fail,
                    step_results: HashMap::new(),
                    variables: self.variable_manager.get_all_variables().clone(),
                    special_vars: HashMap::new(),
                    report_path: None,
                });
            }
        };

        info!("Execution order for {template_id}: {execution_order:?}");

        let mut current_template_step_results: HashMap<
            String,
            crate::template::executor::StepResult,
        > = HashMap::new();
        let mut template_overall_status = StepStatus::Pass;
        let continue_on_error = self.options.as_ref().is_some_and(|o| o.continue_on_error);

        for step_id in execution_order {
            let step_def = match self.step_dependency_manager.get_step(&step_id) {
                Some(s) => s,
                None => {
                    warn!(
                        "Step {step_id} not found in dependency manager, skipping."
                    );
                    continue;
                }
            };

            let parsed_step_details_opt = step_def.original_parsed_step.as_ref();

            let mut step_status = StepStatus::Pass;
            let mut stdout_val = String::new();
            let mut stderr_val = String::new();
            let mut exit_code_val = 0;
            let mut assertion_error_msg: Option<String> = None;
            let mut assertion_status = StepStatus::Skipped;
            let mut assertion_statuses: Vec<StepStatus> = Vec::new();
            let mut assertion_error_msgs: Vec<Option<String>> = Vec::new();

            let step_start_time = Instant::now();

            match &step_def.step_type {
                StepType::CodeBlock {
                    command: cmd_template,
                    ..
                } => {
                    if let Some(parsed_step_details) = parsed_step_details_opt {
                        if parsed_step_details.executable
                            && parsed_step_details.active.unwrap_or(true)
                        {
                            let hydrated_command = self.variable_manager.replace_variables(
                                cmd_template,
                                Some(&step_def.template_id),
                                Some(&step_def.local_id),
                            );
                            debug!(
                                "Executing command for step {step_id}: {hydrated_command}"
                            );

                            let target_config = 
                                &template_arc
                                    .metadata
                                    .target_config;
                            debug!("Executing command on target: {:?}", target_config);
                            let mut current_connection =
                                ConnectionFactory::create_manager(&target_config)?;

                            current_connection.setup()?;

                            let step_timeout_opt =
                                parsed_step_details.timeout_ms.map(Duration::from_millis);
                            let global_timeout_opt = self
                                .options
                                .as_ref()
                                .and_then(|o| o.command_timeout_seconds.map(Duration::from_secs));
                            let timeout_duration = step_timeout_opt.or(global_timeout_opt);

                            match current_connection
                                .execute_command(&hydrated_command, timeout_duration)
                            {
                                Ok(output) => {
                                    stdout_val = output.stdout;
                                    stderr_val = output.stderr;
                                    exit_code_val = output.exit_code;

                                    self.variable_manager.set_variable(
                                        &step_def.template_id,
                                        &step_def.local_id,
                                        "stdout",
                                        &stdout_val,
                                    )?;
                                    // OMG 我们又加了一个硬编码。。
                                    // 新增 stdout_summary 变量，取前 5 行，每行不超过 200 字符，合并为单行
                                    let stdout_summary = {
                                        let mut summary = String::new();
                                        let mut line_count = 0;
                                        for line in stdout_val.lines() {
                                            if line_count >= 5 { break; }
                                            if !summary.is_empty() { summary.push(' '); }
                                            if line.len() > 200 {
                                                summary.push_str(&line[..200]);
                                                summary.push_str("...");
                                            } else {
                                                summary.push_str(line);
                                            }
                                            line_count += 1;
                                        }
                                        if stdout_val.lines().count() > 5 || stdout_val.len() > 200 {
                                            summary.push_str(" ...");
                                        }
                                        summary
                                    };
                                    self.variable_manager.set_variable(
                                        &step_def.template_id,
                                        &step_def.local_id,
                                        "stdout_summary",
                                        &stdout_summary,
                                    )?;
                                    self.variable_manager.set_variable(
                                        &step_def.template_id,
                                        &step_def.local_id,
                                        "stderr",
                                        &stderr_val,
                                    )?;
                                    // 新增 stderr_summary 变量，取前 5 行，每行不超过 200 字符，合并为单行
                                    let stderr_summary = {
                                        let mut summary = String::new();
                                        let mut line_count = 0;
                                        for line in stderr_val.lines() {
                                            if line_count >= 5 { break; }
                                            if !summary.is_empty() { summary.push(' '); }
                                            if line.len() > 200 {
                                                summary.push_str(&line[..200]);
                                                summary.push_str("...");
                                            } else {
                                                summary.push_str(line);
                                            }
                                            line_count += 1;
                                        }
                                        if stderr_val.lines().count() > 5 || stderr_val.len() > 200 {
                                            summary.push_str(" ...");
                                        }
                                        summary
                                    };
                                    self.variable_manager.set_variable(
                                        &step_def.template_id,
                                        &step_def.local_id,
                                        "stderr_summary",
                                        &stderr_summary,
                                    )?;

                                    self.variable_manager.set_variable(
                                        &step_def.template_id,
                                        &step_def.local_id,
                                        "exit_code",
                                        &exit_code_val.to_string(),
                                    )?;

                                    if !parsed_step_details.assertions.is_empty() {
                                        assertion_status = StepStatus::Pass;
                                        for (idx, assertion_details) in
                                            parsed_step_details.assertions.iter().enumerate()
                                        {
                                            let assertion_result = check_assertion(
                                                assertion_details,
                                                &stdout_val,
                                                &stderr_val,
                                                exit_code_val,
                                            );
                                            match assertion_result {
                                                Ok(_) => {
                                                    assertion_statuses.push(StepStatus::Pass);
                                                    assertion_error_msgs.push(None);
                                                }
                                                Err(e) => {
                                                    step_status = StepStatus::Fail;
                                                    assertion_status = StepStatus::Fail;
                                                    assertion_statuses.push(StepStatus::Fail);
                                                    assertion_error_msgs.push(Some(e.to_string()));
                                                    error!(
                                                        "Assertion {idx} failed for step {step_id}: {e}"
                                                    );
                                                }
                                            }
                                        }
                                    } else {
                                        assertion_statuses.clear();
                                        assertion_error_msgs.clear();
                                    }

                                    if step_status == StepStatus::Pass
                                        && !parsed_step_details.extractions.is_empty() {
                                            for extraction_rule in &parsed_step_details.extractions
                                            {
                                                match extract_variable(
                                                    &stdout_val,
                                                    &extraction_rule.regex,
                                                ) {
                                                    Ok(var_value) => {
                                                        debug!(
                                                            "Extracted variable {}={} for step {}",
                                                            extraction_rule.variable,
                                                            var_value,
                                                            step_id
                                                        );
                                                        self.variable_manager.set_variable(
                                                            &step_def.template_id,
                                                            &step_def.local_id,
                                                            &extraction_rule.variable,
                                                            &var_value,
                                                        )?;
                                                    }
                                                    Err(e) => {
                                                        warn!("Failed to extract variable '{}' for step {}: {}", extraction_rule.variable, step_id, e);
                                                    }
                                                }
                                            }
                                        }
                                }
                                Err(e) => {
                                    error!("Command execution failed for step {step_id}: {e}");
                                    step_status = StepStatus::Fail;
                                    assertion_status = StepStatus::Fail;
                                    stderr_val = e.to_string();
                                    assertion_error_msg =
                                        Some(format!("Command execution failed: {e}"));
                                }
                            }
                        } else {
                            info!(
                                "Step {step_id} is inactive or not executable, skipping execution."
                            );
                            step_status = StepStatus::Skipped;
                            assertion_status = StepStatus::Skipped;
                        }
                    } else {
                        error!(
                            "CodeBlock step {step_id} is missing original parsed details. Cannot execute."
                        );
                        step_status = StepStatus::Fail;
                        assertion_status = StepStatus::Fail;
                        stderr_val = format!(
                            "Internal error: CodeBlock {step_id} missing parsed details."
                        );
                        assertion_error_msg = Some(stderr_val.clone());
                    }
                }
                StepType::Heading { .. } => {
                    info!("Skipping execution for heading step: {step_id}");
                    step_status = StepStatus::Skipped;
                    assertion_status = StepStatus::Skipped;
                }
                StepType::OutputPlaceholder => {
                    // OutputPlaceholder steps are handled by the reporter, not executed here.
                    info!("Skipping execution for OutputPlaceholder step: {step_id}");
                    step_status = StepStatus::Skipped;
                    assertion_status = StepStatus::Skipped;
                }
            }

            let duration_ms = step_start_time.elapsed().as_millis();

            if step_status == StepStatus::Fail {
                template_overall_status = StepStatus::Fail;
            }

            let _ = self.variable_manager.set_variable(
                &step_def.template_id,
                &step_def.local_id,
                "status.execution",
                step_status.as_str(),
            );

            let exec_step_result = crate::template::executor::StepResult {
                id: step_def.local_id.clone(),
                description: Some(step_def.description()),
                status: step_status,
                stdout: stdout_val,
                stderr: stderr_val,
                exit_code: exit_code_val,
                duration_ms: Some(duration_ms),
                assertion_error: assertion_error_msg,
            };
            current_template_step_results.insert(
                utils::get_result_id(template_id, step_def.local_id.as_str()),
                exec_step_result.clone(),
            );
            self.executed_step_results
                .insert(step_id.clone(), exec_step_result);

            // 注册断言状态变量（整体）
            let _ = self.variable_manager.set_variable(
                &step_def.template_id,
                &step_def.local_id,
                "status.assertion",
                assertion_status.as_str(),
            );
            // 注册每个断言的状态和错误信息
            for (idx, status) in assertion_statuses.iter().enumerate() {
                let var_name = format!("status.assertion.{idx}");
                let _ = self.variable_manager.set_variable(
                    &step_def.template_id,
                    &step_def.local_id,
                    &var_name,
                    status.as_str(),
                );
                if let Some(Some(err_msg)) = assertion_error_msgs.get(idx) {
                    let err_var_name = format!("assertion_error.{idx}");
                    let _ = self.variable_manager.set_variable(
                        &step_def.template_id,
                        &step_def.local_id,
                        &err_var_name,
                        err_msg,
                    );
                }
            }

            if template_overall_status == StepStatus::Fail && !continue_on_error {
                info!("Stopping execution of template {template_id} due to step failure and continue_on_error=false.");
                break;
            }
        }

        let total_duration_ms = start_time_total.elapsed().as_millis();
        info!(
            "Template {template_id} execution finished in {total_duration_ms} ms. Overall status: {template_overall_status:?}"
        );

        let final_variables = self.variable_manager.get_all_variables().clone();

        let mut execution_result = ExecutionResult {
            template: template_arc.clone(),
            unit_name: template_arc.metadata.unit_name.clone(),
            target_name: template_arc
                .metadata
                .target_config
                .get_name()
                .to_string(),
            overall_status: template_overall_status,
            step_results: current_template_step_results,
            variables: final_variables,
            special_vars: HashMap::new(),
            report_path: None,
        };

        if let Some(report_dir_path) = self.report_dir.as_ref() {
            let template_base_dir = template_arc
                .file_path
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .to_path_buf();
            let reporter = Reporter::new(template_base_dir, Some(report_dir_path.clone()));
            match reporter.generate_report(&template_arc, &execution_result, &self.variable_manager)
            {
                Ok(path) => {
                    info!("Report generated for {template_id}: {path:?}");
                    execution_result.report_path = Some(path);
                }
                Err(e) => {
                    error!("Failed to generate report for {template_id}: {e}");
                }
            }
        } else {
            warn!(
                "Report directory not configured. Skipping report generation for template {template_id}."
            );
        }

        Ok(execution_result)
    }

    pub fn execute_all(&mut self) -> Result<Vec<ExecutionResult>> {
        let mut all_results = Vec::new();
        let all_template_ids: Vec<String> = self.templates.keys().cloned().collect();

        if all_template_ids.is_empty() {
            info!("No templates to execute.");
            return Ok(all_results);
        }

        // Create a name for the virtual summary template
        let summary_template_id = "summary.test.md".to_string();

        for template_id in &all_template_ids {
            match self.execute(template_id) {
                Ok(result) => {
                    all_results.push(result);
                }
                Err(e) => {
                    error!("Failed to execute template {template_id}: {e}. This error will be part of the summary if possible.");
                    // Consider creating a synthetic ExecutionResult for failed templates if needed for summary
                }
            }
        }

        if let Some(report_dir_path) = self.report_dir.as_ref() {
            info!("Generating summary report for all executed templates...");
            if !all_results.is_empty() {
                let summary_file_name = format!("{}.{}", summary_template_id, "report.md");
                let summary_file_path = report_dir_path.join(summary_file_name);
                let mut summary_content = format!(
                    "# Test Execution Summary ({})\n\n",
                    chrono::Local::now().to_rfc3339()
                );
                summary_content.push_str("| Template ID | Overall Status | Steps Passed | Steps Failed | Steps Skipped | Steps Blocked | Steps Not Run | Report File |\n");
                summary_content.push_str("|-------------|----------------|--------------|--------------|---------------|---------------|---------------|-------------|\n");

                for result in &all_results {
                    let mut passed = 0;
                    let mut failed = 0;
                    let mut skipped = 0;
                    let mut blocked = 0;
                    let mut not_run = 0;
                    for step_result in result.step_results.values() {
                        match step_result.status {
                            StepStatus::Pass => passed += 1,
                            StepStatus::Fail => failed += 1,
                            StepStatus::Skipped => skipped += 1,
                            StepStatus::Blocked => blocked += 1,
                            StepStatus::NotRun => not_run += 1,
                        }
                    }
                    let report_link = result
                        .report_path
                        .as_ref()
                        .map(|p| {
                            p.file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .into_owned()
                        })
                        .unwrap_or_else(|| "N/A".to_string());

                    summary_content.push_str(&format!(
                        "| {} | {:?} | {} | {} | {} | {} | {} | {} |\n",
                        result.template.get_template_id(),
                        result.overall_status,
                        passed,
                        failed,
                        skipped,
                        blocked,
                        not_run,
                        report_link
                    ));
                }

                match std::fs::write(&summary_file_path, summary_content) {
                    Ok(_) => info!(
                        "Summary report generated at: {}",
                        summary_file_path.display()
                    ),
                    Err(e) => error!(
                        "Failed to write summary report {}: {}",
                        summary_file_path.display(),
                        e
                    ),
                }
            } else {
                info!("No results to summarize (all template executions might have failed before producing a result object).");
            }
        } else {
            warn!("Report directory not configured. Skipping summary report generation.");
        }

        Ok(all_results)
    }
}
