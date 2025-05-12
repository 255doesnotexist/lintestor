//! 变量管理模块
//! 
//! 这个模块负责处理模板中的变量,包括变量的存储、查找、替换等操作。
//! 提供统一的变量管理接口,处理不同格式的变量引用(命名空间::变量和命名空间.变量)。
//! 支持条件表达式和简单数值比较。

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use log::{debug, info, warn};
use regex::Regex;

/// 变量管理器
/// 
/// 负责变量的存储、查找和替换,提供统一的变量管理接口
#[derive(Debug, Clone)]
pub struct VariableManager {
    /// 变量存储
    variables: HashMap<String, String>,
    
    /// 命名空间到模板ID的映射
    namespace_to_template_id: HashMap<String, String>,
    
    /// 模板路径到模板ID的映射
    template_path_to_id: HashMap<PathBuf, String>,
}

impl VariableManager {
    /// 创建新的变量管理器
    pub fn new() -> Self {
        let mut manager = Self {
            variables: HashMap::new(),
            namespace_to_template_id: HashMap::new(),
            template_path_to_id: HashMap::new(),
        };
        
        // 添加系统预设变量
        let now = chrono::Local::now();
        manager.set_variable("GLOBAL", "GLOBAL", "execution_date", &now.format("%Y-%m-%d").to_string()).unwrap();
        manager.set_variable("GLOBAL", "GLOBAL", "execution_time", &now.format("%H:%M:%S").to_string()).unwrap();
        manager.set_variable("GLOBAL", "GLOBAL", "execution_datetime", &now.format("%Y-%m-%d %H:%M:%S").to_string()).unwrap();
        manager.set_variable("GLOBAL", "GLOBAL", "execution_timestamp", &now.timestamp().to_string()).unwrap();
        
        manager
    }
    
    /// 注册命名空间
    /// 
    /// 将命名空间映射到模板ID
    pub fn register_namespace(&mut self, namespace: &str, template_id: &str) {
        debug!("为命名空间 {} 建立映射到模板ID {}", namespace, template_id);
        self.namespace_to_template_id.insert(namespace.to_string(), template_id.to_string());
    }
    
    /// 注册模板
    /// 
    /// 注册模板路径和对应的模板ID
    pub fn register_template(&mut self, template_path: &Path, template_id: Option<&str>) {
        let template_id = template_id.map(ToString::to_string).unwrap_or_else(|| {
            template_path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string()
        });
        
        self.template_path_to_id.insert(template_path.to_path_buf(), template_id);
    }
    
    /// 设置变量
    /// 
    /// 根据提供的模板ID、步骤ID和变量名存储变量。
    /// 所有标识符部分 (template_id, step_id, name) 必须符合规范:
    /// - 非空
    /// - 不包含 "::"
    /// - template_id 不能以 ".test" 结尾 (特殊值 "GLOBAL" 除外)
    /// 否则将返回错误。
    pub fn set_variable(&mut self, template_id: &str, step_id: &str, name: &str, value: &str) -> Result<(), String> {
        // 1. Validate 'name' (variable part of the key)
        if name.is_empty() {
            return Err(format!("Invalid variable name: '{}'. Cannot be empty.", name));
        }
        if name.contains("::") {
            return Err(format!("Invalid variable name: '{}'. Cannot contain '::'.", name));
        }

        // 2. Validate 'template_id'
        if template_id.is_empty() {
            return Err(format!("Invalid template_id: '{}'. Cannot be empty.", template_id));
        }
        if template_id.contains("::") {
            return Err(format!("Invalid template_id: '{}'. Cannot contain '::'.", template_id));
        }
        // "GLOBAL" is a special keyword and should not be subjected to ".test" suffix check in the same way.
        if template_id != "GLOBAL" && template_id.ends_with(".test") {
            return Err(format!("Invalid template_id: '{}'. Cannot end with '.test'.", template_id));
        }

        // 3. Validate 'step_id'
        if step_id.is_empty() {
            return Err(format!("Invalid step_id: '{}'. Cannot be empty.", step_id));
        }
        if step_id.contains("::") {
            return Err(format!("Invalid step_id: '{}'. Cannot contain '::'.", step_id));
        }
        
        // 生成标准化的变量标识符 (模板ID::步骤ID::变量名)
        let variable_key = format!("{}::{}::{}", template_id, step_id, name);

        debug!("构建变量标识符: {} (template_id='{}', step_id='{}', name='{}')", 
            variable_key, template_id, step_id, name);

        // 使用一个简单的访问记录集合来避免递归引用
        let mut visited_vars = std::collections::HashSet::new();
        if self.has_recursive_reference(&variable_key, value, &mut visited_vars) {
            debug!("检测到递归引用，跳过变量注册: {}", variable_key);
            return Ok(());
        }

        // 检查变量是否已存在
        if self.variables.contains_key(&variable_key) {
            debug!("变量已存在，跳过注册: {}", variable_key);
            return Ok(());
        }

        // 注册变量
        self.variables.insert(variable_key.clone(), value.to_string());
        debug!("注册变量: {} = {}", variable_key, value);
        Ok(())
    }
    
    /// 递归检测变量引用循环
    /// 
    /// 使用BFS算法检测变量值中是否存在递归引用
    fn has_recursive_reference(&self, var_key: &str, value: &str, visited: &mut std::collections::HashSet<String>) -> bool {
        // 如果该变量已经被处理过，说明出现了循环
        if !visited.insert(var_key.to_string()) {
            return true;
        }
        
        // 匹配所有可能的变量引用
        let var_patterns = [
            r"\$\{([a-zA-Z0-9_.:]+)(?:\|([^}]+))?\}", // ${var} 或 ${namespace::var}
            r"\{\{\s*([a-zA-Z0-9_.:]+)\s*\}\}",        // {{ var }} 或 {{ namespace.var }}
            r"\{\s*([a-zA-Z0-9_.:]+)\s*\}"            // { var } 或 { namespace.var }
        ];
        
        for pattern in var_patterns {
            let re = match Regex::new(pattern) {
                Ok(r) => r,
                Err(_) => continue,
            };
            
            for caps in re.captures_iter(value) {
                let referenced_var = caps.get(1).unwrap().as_str();
                
                // 如果引用了自己，那么肯定是循环
                if referenced_var == var_key || referenced_var.ends_with(&format!("::{}", var_key)) {
                    return true;
                }
                
                // 尝试解析引用的变量
                if let Some(referenced_value) = self.get_variable(referenced_var, None, None) {
                    // 递归检查引用的变量
                    if self.has_recursive_reference(referenced_var, &referenced_value, visited) {
                        return true;
                    }
                }
            }
        }
        
        // 没有检测到循环引用
        false
    }
    
    /// 批量设置变量
    /// 
    /// 从提供的映射中为指定的 template_id 和 step_id 设置多个变量。
    /// template_id, step_id, 和映射中的每个键 (作为变量名) 
    /// 都将由内部调用的 `set_variable` 方法进行验证。
    pub fn set_variables_from_map(&mut self, template_id: &str, step_id: &str, variables_map: &HashMap<String, String>) -> Result<(), String> {
        for (key, value) in variables_map {
            // 'key' from variables_map is the 'name' part of the variable.
            // self.set_variable 将验证 template_id, step_id, 和 key (name)。
            self.set_variable(template_id, step_id, key, value)?;
        }
        Ok(())
    }
    
    /// 获取变量值
    /// 
    /// 根据变量名、当前模板ID和步骤ID,按照优先级查找变量值
    pub fn get_variable(&self, var_name: &str, current_template_id: Option<&str>, current_step_id: Option<&str>) -> Option<String> {
        debug!("变量查询: {} (当前模板: {:?}, 当前步骤: {:?})",
            var_name, current_template_id, current_step_id);
        
        // 1. 尝试直接作为完全限定变量名查找
        if let Some(value) = self.variables.get(var_name) {
            debug!("找到完全限定变量: {} = {}", var_name, value);
            return Some(value.clone());
        }
        
        // 2. 处理带命名空间分隔符的变量引用
        if var_name.contains("::") || var_name.contains(".") {
            if let Some(value) = self.resolve_namespaced_variable(var_name) {
                return Some(value);
            }
        }
        
        // 3. 尝试查找带当前上下文前缀的变量
        if let (Some(tid), Some(sid)) = (current_template_id, current_step_id) {
            // 当前代码块的完全限定变量名
            let fully_qualified_name = format!("{}::{}::{}", tid, sid, var_name);
            if let Some(value) = self.variables.get(&fully_qualified_name) {
                debug!("找到当前代码块变量: {} = {}", fully_qualified_name, value);
                return Some(value.clone());
            }
        }
        
        // 4. 尝试查找带当前模板前缀的变量
        if let Some(tid) = current_template_id {
            let template_qualified_name = format!("{}::{}", tid, var_name);
            if let Some(value) = self.variables.get(&template_qualified_name) {
                debug!("找到当前模板变量: {} = {}", template_qualified_name, value);
                return Some(value.clone());
            }
        }
        
        // 5. 最后尝试无前缀变量
        if let Some(value) = self.variables.get(var_name) {
            debug!("找到无前缀全局变量: {} = {}", var_name, value);
            return Some(value.clone());
        }
        
        debug!("未找到变量: {}", var_name);
        None
    }
    
    /// 解析带命名空间的变量引用
    fn resolve_namespaced_variable(&self, var_name: &str) -> Option<String> {
        // 标准化变量名(将.转换为::)
        let normalized_name = var_name.replace('.', "::");
        
        // 解析命名空间和变量名
        let parts: Vec<&str> = normalized_name.splitn(2, "::").collect();
        if parts.len() != 2 {
            return None;
        }
        
        let namespace = parts[0];
        let local_var_name = parts[1];
        
        debug!("解析命名空间变量: 命名空间={}, 变量名={}", namespace, local_var_name);
        
        // 查找命名空间对应的模板ID
        if let Some(template_id) = self.namespace_to_template_id.get(namespace) {
            debug!("命名空间 {} 映射到模板ID {}", namespace, template_id);
            
            // 1. 首先尝试按 template_id::local_var_name 格式查找
            let template_var_name = format!("{}::{}", template_id, local_var_name);
            if let Some(value) = self.variables.get(&template_var_name) {
                debug!("找到命名空间变量: {} = {}", template_var_name, value);
                return Some(value.clone());
            }
            
            // 2. 如果local_var_name中还有::分隔符,说明可能是指向特定步骤的变量
            if local_var_name.contains("::") {
                let full_template_var_name = format!("{}::{}", template_id, local_var_name);
                
                if let Some(value) = self.variables.get(&full_template_var_name) {
                    debug!("找到完全限定命名空间变量: {} = {}", full_template_var_name, value);
                    return Some(value.clone());
                }
            }
        } else {
            // 可能命名空间本身就是一个模板ID,直接尝试查找
            let direct_template_var_name = format!("{}::{}", namespace, local_var_name);
            if let Some(value) = self.variables.get(&direct_template_var_name) {
                debug!("直接找到模板变量: {} = {}", direct_template_var_name, value);
                return Some(value.clone());
            }
            
            warn!("找不到命名空间 {} 对应的模板ID", namespace);
        }
        
        None
    }
    
    /// 获取所有变量
    pub fn get_all_variables(&self) -> &HashMap<String, String> {
        &self.variables
    }
    
    /// 合并另一个变量管理器中的变量
    pub fn merge(&mut self, other: &VariableManager) {
        // 合并变量，确保键名规范化
        for (key, value) in &other.variables {
            // 分析键名，确保不会导致循环嵌套
            let normalized_key = self.normalize_key(key);
            debug!("合并变量：原始键={}, 规范化键={}", key, normalized_key);
            self.variables.insert(normalized_key, value.clone());
        }
        
        // 合并命名空间映射
        for (namespace, template_id) in &other.namespace_to_template_id {
            // 确保模板ID不含分隔符
            let clean_template_id = if template_id.contains("::") {
                warn!("合并命名空间映射时发现模板ID包含'::'分隔符: {}, 进行清理", template_id);
                template_id.split("::").next().unwrap_or(template_id).to_string()
            } else {
                template_id.clone()
            };
            
            self.namespace_to_template_id.insert(namespace.clone(), clean_template_id);
        }
        
        // 合并模板路径映射
        for (path, id) in &other.template_path_to_id {
            // 确保ID不含分隔符
            let clean_id = if id.contains("::") {
                warn!("合并模板路径映射时发现ID包含'::'分隔符: {}, 进行清理", id);
                id.split("::").next().unwrap_or(id).to_string()
            } else {
                id.clone()
            };
            
            self.template_path_to_id.insert(path.clone(), clean_id);
        }
    }
    
    /// 规范化变量键名，避免循环嵌套
    /// 
    /// 分析键名，确保它符合模板ID::步骤ID::变量名的格式，
    /// 移除任何重复的部分(如README::README::var变为README::var)
    pub fn normalize_key(&self, key: &str) -> String {
        // 如果键不包含分隔符，直接返回
        if !key.contains("::") {
            return key.to_string();
        }
        
        let parts: Vec<&str> = key.split("::").collect();
        
        // 如果只有一个或两个部分(如template::var或var)，直接返回
        if parts.len() <= 2 {
            return key.to_string();
        }
        
        // 检查是否存在重复部分（如README::README::var）
        let mut result_parts = Vec::new();
        let mut seen_parts = std::collections::HashSet::new();
        
        // 处理模板ID和步骤ID部分(除了最后一个变量名)
        for part in &parts[..parts.len()-1] {  
            if !seen_parts.contains(*part) {
                seen_parts.insert(*part);
                result_parts.push(*part);
            } else {
                debug!("检测到重复部分在键名中: {}", part);
                // 重复部分不添加
            }
        }
        
        // 添加变量名部分
        result_parts.push(parts[parts.len()-1]);
        
        // 重新组装键名
        let normalized = result_parts.join("::");
        if normalized != key {
            debug!("规范化键名: {} -> {}", key, normalized);
        }
        
        normalized
    }

    /// 替换文本中的变量引用（包括条件表达式）
    ///
    /// 支持多种变量引用格式:
    /// - ${variable_name} - 标准变量引用
    /// - ${namespace::variable_name} - 带命名空间的变量引用
    /// - {{ variable_name }} - 模板风格的双花括号变量引用
    /// - {{ namespace.variable_name }} - 带命名空间的模板风格双花括号变量引用
    /// - { variable_name } - 模板风格的单花括号变量引用
    /// - { namespace.variable_name } - 带命名空间的模板风格单花括号变量引用
    /// - {{ variable == "value" ? "true_result" : "false_result" }} - 三元条件表达式
    /// - {{ variable > 100 ? "high" : "low" }} - 数值比较条件表达式
    pub fn replace_variables(&self, text: &str, current_template_id: Option<&str>, current_step_id: Option<&str>) -> String {
        let mut result = text.to_string();
        
        // 匹配所有标准变量引用 ${variable} 或 ${namespace::variable}
        let var_pattern = r"\$\{([a-zA-Z0-9_.:]+)(?:\|([^}]+))?\}";
        let re = Regex::new(var_pattern).unwrap();
        
        // 匹配双花括号模板风格变量引用 {{ variable }} 或 {{ namespace.variable }}
        let template_pattern = r"\{\{\s*([a-zA-Z0-9_.:]+)\s*\}\}";
        let template_re = Regex::new(template_pattern).unwrap();
        
        // 匹配单花括号模板风格变量引用 { variable } 或 { namespace.variable }
        // 这种单花括号格式用于兼容旧版报告模板
        let single_brace_pattern = r"\{\s*([a-zA-Z0-9_.:]+)\s*\}";
        let single_brace_re = Regex::new(single_brace_pattern).unwrap();
        
        // 匹配三元条件表达式 {{ condition ? true_value : false_value }}
        let conditional_pattern = r"\{\{\s*(.*?)\s*\?\s*(.*?)\s*:\s*(.*?)\s*\}\}";
        let conditional_re = Regex::new(conditional_pattern).unwrap();
        
        // 使用循环而不是单次替换,以处理嵌套变量
        let mut prev_result = String::new();
        let mut iteration = 0;
        let max_iterations = 10; // 防止无限循环
        
        // 跟踪已处理的变量，防止循环引用
        let mut processed_vars = HashMap::new();
        
        while prev_result != result && iteration < max_iterations {
            prev_result = result.clone();
            iteration += 1;
            
            // 每次迭代前重置处理过的变量计数
            processed_vars.clear();
            
            // 1. 处理标准变量引用 ${variable} 或 ${namespace::variable}
            result = re.replace_all(&prev_result, |caps: &regex::Captures| {
                let var_name = &caps[1];
                let default_value = caps.get(2).map(|m| m.as_str()).unwrap_or("");
                
                // 检查循环引用
                let count = processed_vars.entry(var_name.to_string()).or_insert(0);
                *count += 1;
                
                if *count > 3 {  // 如果同一变量被处理超过3次，可能存在循环引用
                    warn!("检测到可能的循环引用: {}", var_name);
                    return if !default_value.is_empty() { 
                        default_value.to_string() 
                    } else { 
                        format!("${{{}}}", var_name) // 保留原始引用
                    };
                }
                
                match self.get_variable(var_name, current_template_id, current_step_id) {
                    Some(value) => value,
                    None if !default_value.is_empty() => default_value.to_string(),
                    None => {
                        // 如果是命名空间变量且无法解析，尝试在日志中提供更多信息
                        if var_name.contains("::") || var_name.contains(".") {
                            let normalized_name = var_name.replace('.', "::");
                            let parts: Vec<&str> = normalized_name.splitn(2, "::").collect();
                            if parts.len() == 2 {
                                let namespace = parts[0];
                                warn!("无法解析命名空间变量: {}，命名空间 {} 未注册或变量不存在", var_name, namespace);
                            }
                        }
                        format!("${{{}}}", var_name) // 保留原始引用
                    }
                }
            }).to_string();
            
            // 重置处理过的变量计数
            processed_vars.clear();
            
            // 2. 处理三元条件表达式 {{ condition ? true_value : false_value }}
            result = conditional_re.replace_all(&result, |caps: &regex::Captures| {
                let condition = &caps[1];
                let true_value = &caps[2];
                let false_value = &caps[3];
                
                // 评估条件
                match self.evaluate_condition(condition, current_template_id, current_step_id) {
                    Ok(true) => true_value.to_string(),
                    Ok(false) => false_value.to_string(),
                    Err(e) => {
                        warn!("条件表达式求值错误: {} - {}", condition, e);
                        format!("{{ {} ? {} : {} }}", condition, true_value, false_value)
                    }
                }
            }).to_string();
            
            // 重置处理过的变量计数
            processed_vars.clear();
            
            // 3. 处理双花括号模板风格变量引用 {{ variable }} 或 {{ namespace.variable }}
            result = template_re.replace_all(&result, |caps: &regex::Captures| {
                let var_name = &caps[1];
                
                // 检查循环引用
                let count = processed_vars.entry(var_name.to_string()).or_insert(0);
                *count += 1;
                
                if *count > 3 {  // 如果同一变量被处理超过3次，可能存在循环引用
                    warn!("检测到可能的循环引用: {}", var_name);
                    return format!("{{ {} }}", var_name); // 保留原始引用
                }
                
                match self.get_variable(var_name, current_template_id, current_step_id) {
                    Some(value) => value,
                    None => {
                        // 如果是命名空间变量且无法解析，尝试在日志中提供更多信息
                        if var_name.contains(".") {
                            let parts: Vec<&str> = var_name.splitn(2, ".").collect();
                            if parts.len() == 2 {
                                let namespace = parts[0];
                                warn!("无法解析命名空间变量: {{ {} }}，命名空间 {} 未注册或变量不存在", var_name, namespace);
                            }
                        }
                        format!("{{ {} }}", var_name) // 保留原始引用
                    }
                }
            }).to_string();
            
            // 重置处理过的变量计数
            processed_vars.clear();
            
            // 4. 处理单花括号模板风格变量引用 { variable } 或 { namespace.variable }
            result = single_brace_re.replace_all(&result, |caps: &regex::Captures| {
                let var_name = &caps[1];
                
                // 检查循环引用
                let count = processed_vars.entry(var_name.to_string()).or_insert(0);
                *count += 1;
                
                if *count > 3 {  // 如果同一变量被处理超过3次，可能存在循环引用
                    warn!("检测到可能的循环引用: {}", var_name);
                    // In case of loop, panic was too strong, return original or placeholder
                    return format!("{{ {} }}", var_name); 
                }
                
                match self.get_variable(var_name, current_template_id, current_step_id) {
                    Some(value) => value,
                    None => {
                        // 如果是命名空间变量且无法解析，尝试在日志中提供更多信息
                        if var_name.contains(".") {
                            let parts: Vec<&str> = var_name.splitn(2, ".").collect();
                            if parts.len() == 2 {
                                let namespace = parts[0];
                                // Corrected warning line:
                                warn!("处理单花括号变量 '{{ {} }}' 时，无法解析其中的命名空间 '{}'。原始引用: '{}'", var_name, namespace, caps.get(0).map_or(var_name, |m| m.as_str()));
                            }
                        }
                        debug!("未找到单花括号变量: {}", var_name);
                        format!("{{ {} }}", var_name) // 保留原始引用，但格式化为双花括号
                    }
                }
            }).to_string();
        }
        
        if iteration >= max_iterations && prev_result != result {
            warn!("变量替换达到最大迭代次数 ({})，可能存在复杂的嵌套或循环引用", max_iterations);
        }
        
        result
    }

    /// 求值条件表达式
    /// 
    /// 支持以下操作:
    /// - 等于: var == value
    /// - 不等于: var != value
    /// - 大于: var > value
    /// - 小于: var < value
    /// - 大于等于: var >= value
    /// - 小于等于: var <= value
    /// - 包含: var contains value
    /// - 不包含: var not_contains value
    /// - 匹配正则: var matches /pattern/
    /// - 不匹配正则: var not_matches /pattern/
    fn evaluate_condition(&self, condition: &str, current_template_id: Option<&str>, current_step_id: Option<&str>) -> Result<bool, String> {
        info!("求值条件表达式: {}", condition);
        let trimmed = condition.trim();
        
        // 处理相等比较 (var == value)
        if let Some(cap) = Regex::new(r"^([^=!<>]+?)\s*==\s*(.+)$").unwrap().captures(trimmed) {
            let left = cap[1].trim();
            let right = cap[2].trim();
            
            // 获取左侧变量值
            let left_value = match self.get_variable(left, current_template_id, current_step_id) {
                Some(value) => value,
                None => return Err(format!("左侧变量不存在: {}", left)),
            };
            
            // 获取右侧值（可能是字面量或变量）
            let right_value = if right.starts_with('"') && right.ends_with('"') {
                // 字面量字符串
                right[1..right.len()-1].to_string()
            } else {
                // 尝试解析为变量
                match self.get_variable(right, current_template_id, current_step_id) {
                    Some(value) => value,
                    None => right.to_string(), // 使用原始值
                }
            };
            
            debug!("比较: '{}' == '{}'", left_value, right_value);
            return Ok(left_value == right_value);
        }
        
        // 处理不等比较 (var != value)
        if let Some(cap) = Regex::new(r"^([^=!<>]+?)\s*!=\s*(.+)$").unwrap().captures(trimmed) {
            let left = cap[1].trim();
            let right = cap[2].trim();
            
            let left_value = match self.get_variable(left, current_template_id, current_step_id) {
                Some(value) => value,
                None => return Err(format!("左侧变量不存在: {}", left)),
            };
            
            let right_value = if right.starts_with('"') && right.ends_with('"') {
                right[1..right.len()-1].to_string()
            } else {
                match self.get_variable(right, current_template_id, current_step_id) {
                    Some(value) => value,
                    None => right.to_string(),
                }
            };
            
            debug!("比较: '{}' != '{}'", left_value, right_value);
            return Ok(left_value != right_value);
        }
        
        // 处理数值比较 (var > value, var < value, var >= value, var <= value)
        if let Some(cap) = Regex::new(r"^([^=!<>]+?)\s*(>=|<=|>|<)\s*(.+)$").unwrap().captures(trimmed) {
            let left = cap[1].trim();
            let op = cap[2].trim();
            let right = cap[3].trim();
            
            let left_value = match self.get_variable(left, current_template_id, current_step_id) {
                Some(value) => value,
                None => return Err(format!("左侧变量不存在: {}", left)),
            };
            
            let right_value = if right.starts_with('"') && right.ends_with('"') {
                right[1..right.len()-1].to_string()
            } else {
                match self.get_variable(right, current_template_id, current_step_id) {
                    Some(value) => value,
                    None => right.to_string(),
                }
            };
            
            // 尝试转换为数值进行比较
            let left_num = match left_value.parse::<f64>() {
                Ok(n) => n,
                Err(_) => return Err(format!("左侧值不是有效数字: {}", left_value)),
            };
            
            let right_num = match right_value.parse::<f64>() {
                Ok(n) => n,
                Err(_) => return Err(format!("右侧值不是有效数字: {}", right_value)),
            };
            
            debug!("数值比较: {} {} {}", left_num, op, right_num);
            
            match op {
                ">" => return Ok(left_num > right_num),
                "<" => return Ok(left_num < right_num),
                ">=" => return Ok(left_num >= right_num),
                "<=" => return Ok(left_num <= right_num),
                _ => return Err(format!("不支持的操作符: {}", op)),
            }
        }
        
        // 处理字符串包含检查 (var contains "value")
        if let Some(cap) = Regex::new(r"^([^=!<>]+?)\s+contains\s+(.+)$").unwrap().captures(trimmed) {
            let left = cap[1].trim();
            let right = cap[2].trim();
            
            let left_value = match self.get_variable(left, current_template_id, current_step_id) {
                Some(value) => value,
                None => return Err(format!("左侧变量不存在: {}", left)),
            };
            
            let right_value = if right.starts_with('"') && right.ends_with('"') {
                right[1..right.len()-1].to_string()
            } else {
                match self.get_variable(right, current_template_id, current_step_id) {
                    Some(value) => value,
                    None => right.to_string(),
                }
            };
            
            debug!("检查包含: '{}' contains '{}'", left_value, right_value);
            return Ok(left_value.contains(&right_value));
        }
        
        // 处理字符串不包含检查 (var not_contains "value")
        if let Some(cap) = Regex::new(r"^([^=!<>]+?)\s+not_contains\s+(.+)$").unwrap().captures(trimmed) {
            let left = cap[1].trim();
            let right = cap[2].trim();
            
            let left_value = match self.get_variable(left, current_template_id, current_step_id) {
                Some(value) => value,
                None => return Err(format!("左侧变量不存在: {}", left)),
            };
            
            let right_value = if right.starts_with('"') && right.ends_with('"') {
                right[1..right.len()-1].to_string()
            } else {
                match self.get_variable(right, current_template_id, current_step_id) {
                    Some(value) => value,
                    None => right.to_string(),
                }
            };
            
            debug!("检查不包含: '{}' not_contains '{}'", left_value, right_value);
            return Ok(!left_value.contains(&right_value));
        }
        
        // 处理正则表达式匹配 (var matches /pattern/)
        if let Some(cap) = Regex::new(r"^([^=!<>]+?)\s+matches\s+/(.+)/$").unwrap().captures(trimmed) {
            let left = cap[1].trim();
            let pattern = cap[2].trim();
            
            let left_value = match self.get_variable(left, current_template_id, current_step_id) {
                Some(value) => value,
                None => return Err(format!("左侧变量不存在: {}", left)),
            };
            
            match Regex::new(pattern) {
                Ok(re) => {
                    debug!("正则匹配: '{}' matches /{}/", left_value, pattern);
                    return Ok(re.is_match(&left_value));
                },
                Err(e) => return Err(format!("无效的正则表达式: {} - {}", pattern, e)),
            }
        }
        
        // 处理正则表达式不匹配 (var not_matches /pattern/)
        if let Some(cap) = Regex::new(r"^([^=!<>]+?)\s+not_matches\s+/(.+)/$").unwrap().captures(trimmed) {
            let left = cap[1].trim();
            let pattern = cap[2].trim();
            
            let left_value = match self.get_variable(left, current_template_id, current_step_id) {
                Some(value) => value,
                None => return Err(format!("左侧变量不存在: {}", left)),
            };
            
            match Regex::new(pattern) {
                Ok(re) => {
                    debug!("正则不匹配: '{}' not_matches /{}/", left_value, pattern);
                    return Ok(!re.is_match(&left_value));
                },
                Err(e) => return Err(format!("无效的正则表达式: {} - {}", pattern, e)),
            }
        }
        
        // 处理布尔值（变量或字面量作为条件）
        if let Some(value) = self.get_variable(trimmed, current_template_id, current_step_id) {
            match value.to_lowercase().as_str() {
                "true" | "yes" | "1" => return Ok(true),
                "false" | "no" | "0" => return Ok(false),
                // 非空字符串视为真
                _ if !value.is_empty() => return Ok(true),
                // 空字符串视为假
                _ => return Ok(false),
            }
        }
        
        Err(format!("无法解析条件表达式: {}", condition))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_variable_operations() {
        let mut manager = VariableManager::new();
        
        // 设置变量
        manager.set_variable("template1", "step1", "test_var", "test_value").unwrap();
        
        // 获取变量
        assert_eq!(manager.get_variable("test_var", Some("template1"), Some("step1")), Some("test_value".to_string()));
        assert_eq!(manager.get_variable("template1::step1::test_var", None, None), Some("test_value".to_string()));
    }
    
    #[test]
    fn test_namespaced_variables() {
        let mut manager = VariableManager::new();
        
        // 注册命名空间
        manager.register_namespace("ns1", "template1");
        
        // 设置变量
        manager.set_variable("template1", "GLOBAL", "var1", "value1").unwrap();
        
        // 通过命名空间访问
        assert_eq!(manager.get_variable("ns1::var1", None, None), Some("value1".to_string()));
        assert_eq!(manager.get_variable("ns1.var1", None, None), Some("value1".to_string()));
    }
    
    #[test]
    fn test_variable_replacement() {
        let mut manager = VariableManager::new();
        
        // 设置变量
        manager.set_variable("template1", "step1", "name", "Alice").unwrap();
        manager.set_variable("template1", "GLOBAL", "greeting", "Hello").unwrap();
        manager.register_namespace("t1", "template1");
        
        // 测试替换
        assert_eq!(
            manager.replace_variables("${greeting} ${name}!", Some("template1"), Some("step1")),
            "Hello Alice!"
        );
        
        assert_eq!(
            manager.replace_variables("{{ t1.greeting }} {{ name }}!", Some("template1"), Some("step1")),
            "Hello Alice!"
        );

        assert_eq!(
            manager.replace_variables("{ t1.greeting } { name }!", Some("template1"), Some("step1")),
            "Hello Alice!"
        );
    }

    #[test]
    fn test_conditional_expressions() {
        let mut manager = VariableManager::new();
        
        // 设置变量
        manager.set_variable("template1", "GLOBAL", "score", "85").unwrap();
        manager.set_variable("template1", "GLOBAL", "name", "Alice").unwrap();
        manager.set_variable("template1", "GLOBAL", "version", "1.2.3").unwrap();
        
        // 测试等于条件
        assert_eq!(
            manager.replace_variables("{{ score == \"85\" ? \"优秀\" : \"良好\" }}", Some("template1"), None),
            "优秀"
        );
        
        // 测试不等于条件
        assert_eq!(
            manager.replace_variables("{{ name != \"Bob\" ? \"不是Bob\" : \"是Bob\" }}", Some("template1"), None),
            "不是Bob"
        );
        
        // 测试大于条件
        assert_eq!(
            manager.replace_variables("{{ score > 80 ? \"优秀\" : \"良好\" }}", Some("template1"), None),
            "优秀"
        );
        
        assert_eq!(
            manager.replace_variables("{{ score > 90 ? \"优秀\" : \"良好\" }}", Some("template1"), None),
            "良好"
        );
        
        // 测试包含条件
        assert_eq!(
            manager.replace_variables("{{ version contains \"1.2\" ? \"1.2系列\" : \"其他版本\" }}", Some("template1"), None),
            "1.2系列"
        );
    }
}