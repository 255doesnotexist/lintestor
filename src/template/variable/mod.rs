//! 变量管理模块
//!
//! 这个模块负责处理模板中的变量,包括变量的存储、查找、替换等操作。
//! 提供统一的变量管理接口,处理不同格式的变量引用(命名空间::变量和命名空间.变量)。
//! 支持条件表达式和简单数值比较。

use log::{debug, info, warn};
use regex::Regex;
use std::collections::HashMap;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::utils;

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
        manager
            .set_variable(
                "GLOBAL",
                "GLOBAL",
                "execution_date",
                &now.format("%Y-%m-%d").to_string(),
            )
            .unwrap();
        manager
            .set_variable(
                "GLOBAL",
                "GLOBAL",
                "execution_time",
                &now.format("%H:%M:%S").to_string(),
            )
            .unwrap();
        manager
            .set_variable(
                "GLOBAL",
                "GLOBAL",
                "execution_datetime",
                &now.format("%Y-%m-%d %H:%M:%S").to_string(),
            )
            .unwrap();
        manager
            .set_variable(
                "GLOBAL",
                "GLOBAL",
                "execution_timestamp",
                &now.timestamp().to_string(),
            )
            .unwrap();

        manager
    }

    /// 注册命名空间
    ///
    /// 将命名空间映射到模板ID
    pub fn register_namespace(&mut self, namespace: &str, template_id: &str) {
        debug!("Establishing mapping for namespace {namespace} to template ID {template_id}"); // 为命名空间 {namespace} 建立映射到模板ID {template_id}
        self.namespace_to_template_id
            .insert(namespace.to_string(), template_id.to_string());
    }

    /// 判断是否存在命名空间
    ///
    /// 检查命名空间是否已注册
    pub fn namespace_exists(&self, _namespace: Option<&str>) -> bool {
        if let Some(namespace) = _namespace {
            self.namespace_to_template_id.contains_key(namespace)
        } else {
            false
        }
    }

    /// 获取命名空间对应的模板ID
    ///
    /// 根据命名空间获取对应的模板ID
    pub fn get_template_id_by_namespace(&self, namespace: &str) -> Option<String> {
        self.namespace_to_template_id.get(namespace).cloned()
    }

    /// 注册模板
    ///
    /// 注册模板路径和对应的模板ID
    pub fn register_template(
        &mut self,
        template: &Arc<super::TestTemplate>,
        template_id: Option<&str>,
    ) -> Result<(), Box<dyn Error>> {
        let template_path = template.file_path.clone();

        let template_id = template_id.map(ToString::to_string).unwrap_or_else(|| {
            template_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string()
        });

        // 注册与模板 id 同名的命名空间
        self.register_namespace(&template_id, &template_id);
        // 其他在这里引用的命名空间（直接导入全局了）
        for item in template.metadata.references.iter() {
            let template_path = item.template_path.clone();
            let as_namespace = item.namespace.clone();
            // 使用 template 自身的 tests_dir 来解析引用的模板ID
            let item_template_id =
                utils::get_template_id_from_path(&template.tests_dir, Path::new(&template_path));
            self.register_namespace(&as_namespace, &item_template_id);
        }

        self.initialize_system_variables(template, template_id.as_str())?;
        self.template_path_to_id
            .insert(template_path.to_path_buf(), template_id);
        Ok(())
    }

    /// 注册时顺便初始化一些变量
    ///
    /// 根据模板ID和步骤ID注册变量
    #[allow(dead_code)] // register_template 里用过了，为什么说没用过
    fn initialize_system_variables(
        &mut self,
        template: &Arc<super::TestTemplate>,
        template_id: &str,
    ) -> Result<(), Box<dyn Error>> {
        // 做一些基本变量的初始化
        // 从这开始 metadata 组变量就可以被使用了
        // 作用域：当前模板、任意步骤、[title, unit, target]
        self.set_variable(
            template_id,
            "GLOBAL",
            "metadata.title",
            &template.metadata.title,
        )?;
        self.set_variable(
            template_id,
            "GLOBAL",
            "metadata.unit_version",
            &template.metadata.unit_version,
        )?;
        self.set_variable(
            template_id,
            "GLOBAL",
            "metadata.unit_name",
            &template.metadata.unit_name,
        )?;
        self.set_variable(
            template_id,
            "GLOBAL",
            "metadata.target_name",
            template.metadata.target_config.get_name(),
        )?;
        self.set_variable(
            template_id,
            "GLOBAL",
            "metadata.target_description",
            template.metadata.target_config.get_description(),
        )?;
        Ok(())
    }

    /// 设置变量
    ///
    /// 根据提供的模板ID、步骤ID和变量名存储变量。
    /// 所有标识符部分 (template_id, step_id, name) 必须符合规范:
    /// - 非空
    /// - 不包含 "::"
    /// - template_id 不能以 ".test" 结尾 (特殊值 "GLOBAL" 除外)
    ///   否则将返回错误。
    pub fn set_variable(
        &mut self,
        _template_id: &str,
        step_id: &str,
        name: &str,
        value: &str,
    ) -> Result<(), String> {
        // 1. Validate 'name' (variable part of the key)
        if name.is_empty() {
            return Err(format!("Invalid variable name: '{name}'. Cannot be empty."));
        }
        if name.contains("::") {
            return Err(format!(
                "Invalid variable name: '{name}'. Cannot contain '::'."
            ));
        }

        // 2. Validate 'template_id'
        let mut template_id = _template_id.to_string();
        // 来点命名空间适配，希望没有什么会在这里造成死循环
        while self.namespace_exists(Some(template_id.as_str()))
            && !self.template_id_exists(template_id.as_str())
        {
            // 如果 template_id 是命名空间，获取对应的模板ID
            if let Some(tid) = self.get_template_id_by_namespace(template_id.as_str()) {
                warn!(
                    "Theoretically you shouldn't use namespace instead of specific ID during assignment, but namespace {template_id} is automatically resolved to template ID {tid} here"
                ); // 理论上你不该在赋值时还用的命名空间而不是具体 ID，但是命名空间 {template_id} 这里自动被解析为模板ID {tid}
                if template_id == tid {
                    warn!(
                        "Namespace {template_id} and template ID {tid} are the same, avoiding infinite loop, breaking out"
                    ); // 命名空间 {template_id} 和模板ID {tid} 相同，避免死循环，跳出
                    break; // 避免死循环
                }
                template_id = tid;
            } else {
                break; // 没有对应的模板ID，退出循环
            }
        }
        if template_id.is_empty() {
            return Err(format!(
                "Invalid template_id: '{template_id}'. Cannot be empty."
            ));
        }
        if template_id.contains("::") {
            return Err(format!(
                "Invalid template_id: '{template_id}'. Cannot contain '::'."
            ));
        }
        // "GLOBAL" is a special keyword and should not be subjected to ".test" suffix check in the same way.
        // 只觉得这行非常没意义但反正没有副作用。。
        if template_id != "GLOBAL" && template_id.ends_with(".test") {
            return Err(format!(
                "Invalid template_id: '{template_id}'. Cannot end with '.test'."
            ));
        }

        // 3. Validate 'step_id'
        if step_id.is_empty() {
            return Err(format!("Invalid step_id: '{step_id}'. Cannot be empty."));
        }
        if step_id.contains("::") {
            return Err(format!(
                "Invalid step_id: '{step_id}'. Cannot contain '::'."
            ));
        }

        // 生成标准化的变量标识符 (模板ID::步骤ID::变量名)
        let variable_key = format!("{template_id}::{step_id}::{name}");

        debug!(
            "Building variable identifier: {variable_key} (template_id='{template_id}', step_id='{step_id}', name='{name}')" // 构建变量标识符: {variable_key} (template_id='{template_id}', step_id='{step_id}', name='{name}')
        );

        // 使用一个简单的访问记录集合来避免递归引用
        let mut visited_vars = std::collections::HashSet::new();
        if self.has_recursive_reference(&variable_key, value, &mut visited_vars) {
            debug!("Detected recursive reference, skipping variable registration: {variable_key}"); // 检测到递归引用，跳过变量注册: {variable_key}
            return Ok(());
        }

        // 检查变量是否已存在
        if self.variables.contains_key(&variable_key) {
            debug!("Variable already exists, overwriting old value: {variable_key}"); // 变量已存在，覆盖旧值: {variable_key}
            self.variables
                .insert(variable_key.clone(), value.to_string());
        }

        // 注册变量
        self.variables
            .insert(variable_key.clone(), value.to_string());
        debug!("Registered variable: {variable_key} = {value}"); // 注册变量: {variable_key} = {value}
        Ok(())
    }

    /// 递归检测变量引用循环
    ///
    /// 使用BFS算法检测变量值中是否存在递归引用
    fn has_recursive_reference(
        &self,
        var_key: &str,
        value: &str,
        visited: &mut std::collections::HashSet<String>,
    ) -> bool {
        // 如果该变量已经被处理过，说明出现了循环
        if !visited.insert(var_key.to_string()) {
            return true;
        }

        // 匹配所有可能的变量引用
        let var_patterns = [
            r"\$\{([a-zA-Z0-9_.:]+)(?:\|([^}]+))?\}", // ${var} 或 ${namespace::var}
            r"\{\{\s*([a-zA-Z0-9_.:]+)\s*\}\}",       // {{ var }} 或 {{ namespace.var }}
            r"\{\s*([a-zA-Z0-9_.:]+)\s*\}",           // { var } 或 { namespace.var }
        ];

        for pattern in var_patterns {
            let re = match Regex::new(pattern) {
                Ok(r) => r,
                Err(_) => continue,
            };

            for caps in re.captures_iter(value) {
                let referenced_var = caps.get(1).unwrap().as_str();

                // 如果引用了自己，那么肯定是循环
                if referenced_var == var_key || referenced_var.ends_with(&format!("::{var_key}")) {
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
    #[allow(dead_code)]
    pub fn set_variables_from_map(
        &mut self,
        template_id: &str,
        step_id: &str,
        variables_map: &HashMap<String, String>,
    ) -> Result<(), String> {
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
    pub fn get_variable(
        &self,
        _var_name: &str,
        _current_template_id: Option<&str>,
        _current_step_id: Option<&str>,
    ) -> Option<String> {
        // 以下俩 ID 等待回填中，填完了要说还是 UNFILLED 该 panic 了
        let mut current_template_id = "UNFILLED".to_string();
        let mut current_step_id = "UNFILLED".to_string();
        let mut var_name = _var_name.to_string();
        // 如果用户传入的 var_name 已经指定了完整命名空间怎么办呢？
        // 首先 { t1::step::greeting } 里，t1 和 step 是不会体现在传入的 _current 里的
        // 因为传入的这个是当前代码块所处的相对 template_id 和 step_id
        // 因此，一旦 var_name 里有 ::，我们就要手动提取命名空间和步骤 ID
        // 为了偷懒少写逻辑，接下来可以直接用提取到的 ns 和 stp_id override 掉原本的 _current 系列变量
        // 另外，还有两种情况：用户可能传入 t1::greeting 或 t1::step::greeting
        // 前者意味着 t1::GLOBAL::greeting 的语法糖，我们也要能妥善处理
        // 这里是一个 workaround
        if var_name.contains("::") {
            let parts: Vec<String> = var_name.splitn(3, "::").map(|s| s.to_string()).collect();
            if parts.len() == 3 {
                // 完整的命名空间::步骤::变量名
                current_template_id = parts[0].clone();
                current_step_id = parts[1].clone();
                var_name = parts[2].clone();
            } else if parts.len() == 2 {
                let first = &parts[0];
                let second = &parts[1];
                // 如果first是命名空间/模板ID，补全为 first::GLOBAL::second
                // 我其实觉得在这里引入如此有二义性的逻辑是个错误
                // 但是为了对某些模板的兼容性就先这样吧
                // TODO: 考虑是否需要在这里引入更严格的检查
                if self.namespace_exists(Some(first)) || self.template_id_exists(first) {
                    current_template_id = first.clone();
                    current_step_id = "GLOBAL".to_string();
                    var_name = second.clone();
                } else {
                    // 否则，自动补全为当前template_id::first::second
                    current_step_id = first.clone();
                    var_name = second.clone();
                    // current_template_id 保持不变
                }
            }
        }

        // 这之后，还是先转一波第一位的标识符，看看如果是命名空间的话需不需要转模板 ID 哈。
        if current_template_id == "UNFILLED" {
            // 如果用户没在 var_name 里指定 template_id，隐式地使用当前代码块处于的 template_id
            current_template_id = _current_template_id.unwrap_or("GLOBAL").to_string();
        }
        // 来点命名空间适配，希望没有什么会在这里造成死循环
        while self.namespace_exists(Some(current_template_id.as_str()))
            && !self.template_id_exists(current_template_id.as_str())
        {
            // 如果 template_id 是命名空间，获取对应的模板ID
            if let Some(tid) = self.get_template_id_by_namespace(current_template_id.as_str()) {
                debug!("Namespace {current_template_id} resolved to template ID {tid}"); // 命名空间 {current_template_id} 被解析为模板ID {tid}
                if current_template_id == tid {
                    warn!(
                        "Namespace {current_template_id} and template ID {tid} are the same, avoiding infinite loop"
                    ); // 命名空间 {current_template_id} 和模板ID {tid} 相同，避免死循环，跳出
                    break; // 避免死循环
                }
                current_template_id = tid;
            } else {
                break; // 没有对应的模板ID，退出循环
            }
        }

        if current_step_id == "UNFILLED" {
            // 如果用户没在 var_name 里指定 step_id，隐式地使用当前代码块处于的 step_id
            current_step_id = _current_step_id.unwrap_or("GLOBAL").to_string();
        }
        // 这里的 current_template_id 和 current_step_id 已经是经过处理的
        debug!(
            "Variable query: '{var_name}' (current template: {current_template_id:?}, current step: {current_step_id:?})" // 变量查询: '{var_name}' (当前模板: {current_template_id:?}, 当前步骤: {current_step_id:?})
        );

        if current_template_id == "GLOBAL" && current_step_id != "GLOBAL" {
            panic!("Variable reference format /wildcard_namespace/::step::var_name is not allowed"); // 不允许 /wildcard_namespace/::step::var_name 这种格式的变量引用
        }

        // 变量名允许带点（如 status.execution），查找时整体作为变量名处理，不做特殊分割

        // 1. 尝试直接作为完全限定变量名查找 (e.g., "T1::S1::V1", "GLOBAL::GLOBAL::V1", "T1::GLOBAL::V1")
        // 或简单名称（如果它们是这样存储的，例如旧版全局变量）
        if let Some(value) = self.variables.get(&var_name) {
            debug!("Direct match found variable '{var_name}': {value}"); // 直接匹配找到变量 '{var_name}': {value}
            return Some(value.clone());
        }

        // 这里我在前面用简单逻辑做了 workaround
        // // 2. 处理带命名空间分隔符的变量引用 (e.g., "NS::V", "NS::S::V")
        // if var_name.contains("::") {
        //     // 主动检查不允许的模式: GLOBAL::SpecificStep::Var from non-GLOBAL template context
        //     let parts: Vec<&str> = var_name.splitn(3, "::").collect();
        //     // 检查是否为 "GLOBAL::NotGlobalStep::VarName" 格式
        //     if parts.len() == 3 && parts[0] == "GLOBAL" && parts[1] != "GLOBAL" {
        //         if current_template_id != "GLOBAL" {
        //             warn!("不允许的变量查询: 从模板 '{}' 查询 '{}'。不允许从非全局模板通过 'GLOBAL::SpecificStep::VarName' 格式引用特定步骤变量。", current_template_id, var_name);
        //             return None; // 主动禁止
        //         }
        //     }
        // }

        // 3. 上下文查找 (此时 var_name 是简单的, 例如 "query", 不包含 "::")
        let (tid, sid) = (current_template_id.clone(), current_step_id.clone());
        // a. tid::sid::var_name (例如 current_namespace::current_step::query)
        let key1 = format!("{tid}::{sid}::{var_name}");
        if let Some(value) = self.variables.get(&key1) {
            debug!("Found variable ({key1}): {value}"); // 找到变量 ({key1}): {value}
            return Some(value.clone());
        }

        // b. tid::GLOBAL::var_name (例如 current_namespace::GLOBAL::query)
        //    仅当 sid 不是 "GLOBAL" 时尝试，以避免与 key1 重复检查
        if sid != "GLOBAL" {
            let key2 = format!("{}::{}::{}", tid, "GLOBAL", var_name);
            if let Some(value) = self.variables.get(&key2) {
                debug!("Found variable ({key2}): {value}"); // 找到变量 ({key2}): {value}
                return Some(value.clone());
            }
        }
        // 如果 sid == "GLOBAL", key1 已经是 tid::GLOBAL::var_name, 所以 key2 被跳过。

        // c. GLOBAL::GLOBAL::var_name (例如 GLOBAL::GLOBAL::query)
        let mut try_key3 = true;
        let key3 = format!("{}::{}::{}", "GLOBAL", "GLOBAL", var_name);

        // 如果 key1 已经是 GLOBAL::GLOBAL::var_name (当 tid="GLOBAL" 且 sid="GLOBAL")
        if tid == "GLOBAL" && sid == "GLOBAL" {
            try_key3 = false;
        }
        // 如果 key2 已经是 GLOBAL::GLOBAL::var_name (当 tid="GLOBAL" 且 sid!="GLOBAL", key2尝试了GLOBAL::GLOBAL::var_name)
        if tid == "GLOBAL" && sid != "GLOBAL" {
            // key1 是 GLOBAL::sid::var_name, key2 是 GLOBAL::GLOBAL::var_name
            try_key3 = false;
        }
        // 因此，仅当 tid 不是 "GLOBAL" 时，才需要独立尝试 key3

        if try_key3 {
            // 这意味着 tid 不是 "GLOBAL"
            if let Some(value) = self.variables.get(&key3) {
                debug!("Found variable ({key3}): {value}"); // 找到变量 ({key3}): {value}
                return Some(value.clone());
            }
        }

        debug!("Variable not found '{var_name}'"); // 未找到变量 '{var_name}'
        None
    }

    /// 获取所有变量
    pub fn get_all_variables(&self) -> &HashMap<String, String> {
        &self.variables
    }

    /// 检查模板ID是否存在
    pub fn template_id_exists(&self, template_id: &str) -> bool {
        self.template_path_to_id
            .values()
            .any(|id| id == template_id)
    }

    /// 替换文本中的变量引用（包括条件表达式）
    ///
    /// 支持多种变量引用格式: (OUTDATED maybe, need updates)
    /// - ${variable_name} - 标准变量引用
    /// - ${namespace::variable_name} - 带命名空间的变量引用
    /// - {{ variable_name }} - 模板风格的双花括号变量引用
    /// - {{ namespace.variable_name }} - 带命名空间的模板风格双花括号变量引用
    /// - { variable_name } - 模板风格的单花括号变量引用
    /// - { namespace.variable_name } - 带命名空间的模板风格单花括号变量引用
    ///   必须有两个 { 和 } 才是三元表达式
    /// - {{ variable == "value" ? "true_result" : "false_result" }} - 三元条件表达式
    /// - {{ variable > 100 ? "high" : "low" }} - 数值比较条件表达式
    pub fn replace_variables(
        &self,
        text: &str,
        input_ns_or_template_id: Option<&str>,
        current_step_id: Option<&str>,
    ) -> String {
        // 输入的可能是命名空间也可能是模板ID
        let current_template_id = if self.namespace_exists(input_ns_or_template_id) {
            self.get_template_id_by_namespace(input_ns_or_template_id.unwrap())
                .unwrap_or_else(|| input_ns_or_template_id.unwrap_or("GLOBAL").to_string())
        } else if self.template_id_exists(input_ns_or_template_id.unwrap_or("GLOBAL")) {
            input_ns_or_template_id.unwrap_or("GLOBAL").to_string()
        } else {
            if input_ns_or_template_id.is_some() {
                warn!(
                    "This namespace or template ID appears to not be registered, please check: {}", // 这个命名空间或者模板 ID 疑似没有被注册，请检查: {}
                    input_ns_or_template_id.unwrap_or("GLOBAL")
                );
            }
            "GLOBAL".to_string()
        };

        let current_step_id = current_step_id.unwrap_or("GLOBAL");

        let mut result = text.to_string();

        #[derive(Debug, PartialEq)]
        enum State {
            Normal,
            Escape,
            VarDollarContent { content: String, brace_level: usize },
            VarDoubleBraceContent { content: String },
            VarSingleBraceContent { content: String, brace_level: usize },
        }

        let mut chars = result.chars().peekable();
        let mut output = String::new();
        let mut state = State::Normal;

        while let Some(c) = chars.next() {
            match &mut state {
                State::Normal => {
                    if c == '\\' {
                        state = State::Escape;
                    } else if c == '$' && chars.peek() == Some(&'{') {
                        chars.next(); // 跳过 '{'
                        state = State::VarDollarContent {
                            content: String::new(),
                            brace_level: 1,
                        };
                    } else if c == '{' && chars.peek() == Some(&'{') {
                        chars.next(); // 跳过第二个 '{'
                        state = State::VarDoubleBraceContent {
                            content: String::new(),
                        };
                    } else if c == '{' {
                        state = State::VarSingleBraceContent {
                            content: String::new(),
                            brace_level: 1,
                        };
                    } else {
                        output.push(c);
                    }
                }
                State::Escape => {
                    // 只转义变量包裹符
                    if c == '$' || c == '{' {
                        output.push(c);
                    } else {
                        output.push('\\');
                        output.push(c);
                    }
                    state = State::Normal;
                }
                State::VarDollarContent {
                    content,
                    brace_level,
                } => {
                    if c == '\n' || c == '\r' {
                        // 换行符，降级为原文输出
                        output.push_str(&format!("${{{content}}}"));
                        output.push(c);
                        state = State::Normal;
                    } else if c == '{' {
                        *brace_level += 1;
                        content.push(c);
                    } else if c == '}' {
                        *brace_level -= 1;
                        if *brace_level == 0 {
                            // 处理 ${...}
                            let mut parts = content.splitn(2, '|');
                            let var_name = parts.next().unwrap_or("").trim();
                            let default_value = parts.next().unwrap_or("");
                            let value = match self.get_variable(
                                var_name,
                                Some(current_template_id.as_str()),
                                Some(current_step_id),
                            ) {
                                Some(v) => v,
                                None if !default_value.is_empty() => default_value.to_string(),
                                None => format!("${{{var_name}}}"),
                            };
                            output.push_str(&value);
                            state = State::Normal;
                        } else {
                            content.push(c);
                        }
                    } else {
                        content.push(c);
                    }
                }
                State::VarDoubleBraceContent { content } => {
                    if c == '\n' || c == '\r' {
                        // 换行符，降级为原文输出
                        output.push_str(&format!("{{{{ {content} }}}}"));
                        output.push(c);
                        state = State::Normal;
                    } else if c == '}' && chars.peek() == Some(&'}') {
                        chars.next(); // 跳过第二个 '}'
                        // Unicode安全：用char_indices记录?和:的字节下标
                        let inner = content.trim();
                        let mut qmark_byte = None;
                        let mut colon_byte = None;
                        let mut level = 0;
                        for (i, ch) in inner.char_indices() {
                            if ch == '?' && level == 0 && qmark_byte.is_none() {
                                qmark_byte = Some(i);
                            } else if ch == ':'
                                && level == 0
                                && colon_byte.is_none()
                                && qmark_byte.is_some()
                            {
                                colon_byte = Some(i);
                            } else if ch == '{' {
                                level += 1;
                            } else if ch == '}' && level > 0 {
                                level -= 1;
                            }
                        }
                        if let (Some(q), Some(c)) = (qmark_byte, colon_byte) {
                            // 三元表达式
                            let cond = inner[..q].trim();
                            let tval = inner[q + 1..c].trim();
                            let fval = inner[c + 1..].trim();
                            let mut output_val = match self.evaluate_condition(
                                cond,
                                Some(current_template_id.as_str()),
                                Some(current_step_id),
                            ) {
                                Ok(true) => tval.to_string(),
                                Ok(false) => fval.to_string(),
                                Err(e) => {
                                    warn!("Conditional expression evaluation error: {cond} - {e}"); // 条件表达式求值错误: {cond} - {e}
                                    format!("{{{{ {cond} ? {tval} : {fval} }}}}")
                                }
                            };
                            if output_val.starts_with('"')
                                && output_val.ends_with('"')
                                && output_val.len() > 1
                            {
                                output_val = output_val[1..output_val.len() - 1].to_string();
                            } else {
                                output_val = self.replace_variables(
                                    &output_val,
                                    Some(current_template_id.as_str()),
                                    Some(current_step_id),
                                );
                            }
                            output.push_str(&output_val);
                        } else {
                            // 普通变量
                            let var_name = inner;
                            let value = match self.get_variable(
                                var_name,
                                Some(current_template_id.as_str()),
                                Some(current_step_id),
                            ) {
                                Some(v) => v,
                                None => format!("{{{{ {var_name} }}}}"),
                            };
                            output.push_str(&value);
                        }
                        state = State::Normal;
                    } else {
                        content.push(c);
                    }
                }
                State::VarSingleBraceContent {
                    content,
                    brace_level,
                } => {
                    if c == '\n' || c == '\r' {
                        // 换行符，降级为原文输出
                        output.push_str(&format!("{{ {content} }}"));
                        output.push(c);
                        state = State::Normal;
                    } else if c == '{' {
                        *brace_level += 1;
                        content.push(c);
                    } else if c == '}' {
                        *brace_level -= 1;
                        if *brace_level == 0 {
                            // 处理 { ... }
                            let var_name = content.trim();
                            let value = match self.get_variable(
                                var_name,
                                Some(current_template_id.as_str()),
                                Some(current_step_id),
                            ) {
                                Some(v) => v,
                                None => format!("{{ {var_name} }}"),
                            };
                            output.push_str(&value);
                            state = State::Normal;
                        } else {
                            content.push(c);
                        }
                    } else {
                        content.push(c);
                    }
                }
            }
        }
        // 如果结束时还在变量状态，降级为原文输出
        match state {
            State::VarDollarContent { content, .. } => {
                output.push_str(&format!("${{{content}}}"));
            }
            State::VarDoubleBraceContent { content } => {
                output.push_str(&format!("{{{{ {content} }}}}"));
            }
            State::VarSingleBraceContent { content, .. } => {
                output.push_str(&format!("{{ {content} }}"));
            }
            State::Escape => {
                output.push('\\');
            }
            _ => {}
        }
        result = output;

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
    fn evaluate_condition(
        &self,
        condition: &str,
        current_template_id: Option<&str>,
        current_step_id: Option<&str>,
    ) -> Result<bool, String> {
        info!("Evaluating conditional expression: {condition}"); // 求值条件表达式: {condition}
        let trimmed = condition.trim();

        // 处理相等比较 (var == value)
        if let Some(cap) = Regex::new(r"^([^=!<>]+?)\s*==\s*(.+)$")
            .unwrap()
            .captures(trimmed)
        {
            let left = cap[1].trim();
            let right = cap[2].trim();

            // 获取左侧值（可能是字面量或变量）
            let left_value = if left.starts_with('"') && left.ends_with('"') {
                // 字面量字符串，去掉引号
                left[1..left.len() - 1].to_string()
            } else {
                // 尝试解析为变量
                match self.get_variable(left, current_template_id, current_step_id) {
                    Some(value) => value,
                    None => left.to_string(), // 使用原始值
                }
            };

            // 获取右侧值（可能是字面量或变量）
            let right_value = if right.starts_with('"') && right.ends_with('"') {
                // 字面量字符串，去掉引号
                right[1..right.len() - 1].to_string()
            } else {
                // 尝试解析为变量
                match self.get_variable(right, current_template_id, current_step_id) {
                    Some(value) => value,
                    None => right.to_string(), // 使用原始值
                }
            };

            debug!("Comparing: '{left_value}' == '{right_value}'"); // 比较: '{left_value}' == '{right_value}'
            return Ok(left_value == right_value);
        }

        // 处理不等比较 (var != value)
        if let Some(cap) = Regex::new(r"^([^=!<>]+?)\s*!=\s*(.+)$")
            .unwrap()
            .captures(trimmed)
        {
            let left = cap[1].trim();
            let right = cap[2].trim();

            let left_value = match self.get_variable(left, current_template_id, current_step_id) {
                Some(value) => value,
                None => return Err(format!("Left variable does not exist: {left}")), // 左侧变量不存在: {left}
            };

            let right_value = if right.starts_with('"') && right.ends_with('"') {
                right[1..right.len() - 1].to_string()
            } else {
                match self.get_variable(right, current_template_id, current_step_id) {
                    Some(value) => value,
                    None => right.to_string(),
                }
            };

            debug!("Comparison: '{left_value}' != '{right_value}'"); // 比较: '{left_value}' != '{right_value}'
            return Ok(left_value != right_value);
        }

        // 处理数值比较 (var > value, var < value, var >= value, var <= value)
        if let Some(cap) = Regex::new(r"^([^=!<>]+?)\s*(>=|<=|>|<)\s*(.+)$")
            .unwrap()
            .captures(trimmed)
        {
            let left = cap[1].trim();
            let op = cap[2].trim();
            let right = cap[3].trim();

            let left_value = match self.get_variable(left, current_template_id, current_step_id) {
                Some(value) => value,
                None => return Err(format!("Left variable does not exist: {left}")), // 左侧变量不存在: {left}
            };

            let right_value = if right.starts_with('"') && right.ends_with('"') {
                right[1..right.len() - 1].to_string()
            } else {
                match self.get_variable(right, current_template_id, current_step_id) {
                    Some(value) => value,
                    None => right.to_string(),
                }
            };

            // 尝试转换为数值进行比较
            let left_num = match left_value.parse::<f64>() {
                Ok(n) => n,
                Err(_) => return Err(format!("Left value is not a valid number: {left_value}")), // 左侧值不是有效数字: {left_value}
            };

            let right_num = match right_value.parse::<f64>() {
                Ok(n) => n,
                Err(_) => return Err(format!("Right value is not a valid number: {right_value}")), // 右侧值不是有效数字: {right_value}
            };

            debug!("Numerical comparison: {left_num} {op} {right_num}"); // 数值比较: {left_num} {op} {right_num}

            match op {
                ">" => return Ok(left_num > right_num),
                "<" => return Ok(left_num < right_num),
                ">=" => return Ok(left_num >= right_num),
                "<=" => return Ok(left_num <= right_num),
                _ => return Err(format!("Unsupported operator: {op}")), // 不支持的操作符: {op}
            }
        }

        // 处理字符串包含检查 (var contains "value")
        if let Some(cap) = Regex::new(r"^([^=!<>]+?)\s+contains\s+(.+)$")
            .unwrap()
            .captures(trimmed)
        {
            let left = cap[1].trim();
            let right = cap[2].trim();

            let left_value = match self.get_variable(left, current_template_id, current_step_id) {
                Some(value) => value,
                None => return Err(format!("Left variable does not exist: {left}")), // 左侧变量不存在: {left}
            };

            let right_value = if right.starts_with('"') && right.ends_with('"') {
                right[1..right.len() - 1].to_string()
            } else {
                match self.get_variable(right, current_template_id, current_step_id) {
                    Some(value) => value,
                    None => right.to_string(),
                }
            };

            debug!("Check contains: '{left_value}' contains '{right_value}'"); // 检查包含: '{left_value}' contains '{right_value}'
            return Ok(left_value.contains(&right_value));
        }

        // 处理字符串不包含检查 (var not_contains "value")
        if let Some(cap) = Regex::new(r"^([^=!<>]+?)\s+not_contains\s+(.+)$")
            .unwrap()
            .captures(trimmed)
        {
            let left = cap[1].trim();
            let right = cap[2].trim();

            let left_value = match self.get_variable(left, current_template_id, current_step_id) {
                Some(value) => value,
                None => return Err(format!("Left variable does not exist: {left}")), // 左侧变量不存在: {left}
            };

            let right_value = if right.starts_with('"') && right.ends_with('"') {
                right[1..right.len() - 1].to_string()
            } else {
                match self.get_variable(right, current_template_id, current_step_id) {
                    Some(value) => value,
                    None => right.to_string(),
                }
            };

            debug!("Check not contains: '{left_value}' not_contains '{right_value}'"); // 检查不包含: '{left_value}' not_contains '{right_value}'
            return Ok(!left_value.contains(&right_value));
        }

        // 处理正则表达式匹配 (var matches /pattern/)
        if let Some(cap) = Regex::new(r"^([^=!<>]+?)\s+matches\s+/(.+)/$")
            .unwrap()
            .captures(trimmed)
        {
            let left = cap[1].trim();
            let pattern = cap[2].trim();

            let left_value = match self.get_variable(left, current_template_id, current_step_id) {
                Some(value) => value,
                None => return Err(format!("Left variable does not exist: {left}")), // 左侧变量不存在: {left}
            };

            match Regex::new(pattern) {
                Ok(re) => {
                    debug!("Regex match: '{left_value}' matches /{pattern}/"); // 正则匹配: '{left_value}' matches /{pattern}/
                    return Ok(re.is_match(&left_value));
                }
                Err(e) => return Err(format!("Invalid regex: {pattern} - {e}")), // 无效的正则表达式: {pattern} - {e}
            }
        }

        // 处理正则表达式不匹配 (var not_matches /pattern/)
        if let Some(cap) = Regex::new(r"^([^=!<>]+?)\s+not_matches\s+/(.+)/$")
            .unwrap()
            .captures(trimmed)
        {
            let left = cap[1].trim();
            let pattern = cap[2].trim();

            let left_value = match self.get_variable(left, current_template_id, current_step_id) {
                Some(value) => value,
                None => return Err(format!("Left variable does not exist: {left}")), // 左侧变量不存在: {left}
            };

            match Regex::new(pattern) {
                Ok(re) => {
                    debug!("Regex not match: '{left_value}' not_matches /{pattern}/"); // 正则不匹配: '{left_value}' not_matches /{pattern}/
                    return Ok(!re.is_match(&left_value));
                }
                Err(e) => return Err(format!("Invalid regex: {pattern} - {e}")), // 无效的正则表达式: {pattern} - {e}
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

        Err(format!(
            "Unable to parse conditional expression: {condition}"
        )) // 无法解析条件表达式: {condition}
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        config::target_config::TargetConfig,
        template::{
            ContentBlock, ExecutionStep, TemplateMetadata, TemplateReference, TestTemplate,
        },
    };

    use super::*;

    fn create_dummy_template(
        id: &str,
        raw_content: &str,
        content_blocks: Vec<ContentBlock>,
        steps: Vec<ExecutionStep>,
    ) -> Arc<TestTemplate> {
        // 这里可以根据需要构造 TemplateMetadata、steps 等
        // 但通常测试用例只关心 content_blocks

        // 写入 /tmp/dummy_target.toml，内容为本地测试
        let dummy_target_path = "/tmp/dummy_target.toml";
        let dummy_target_content = r#"
    name = "Local Test" # 本地测试
    testing_type = "local"
    description = "Local Test" # 本地测试
    "#;
        let _ = std::fs::write(dummy_target_path, dummy_target_content);

        Arc::new(TestTemplate {
            file_path: PathBuf::from(format!("/test/{}.test.md", id)),
            tests_dir: PathBuf::from("/test/"),
            raw_content: raw_content.to_string(),
            content_blocks,
            metadata: TemplateMetadata {
                title: format!("{} Title", id),
                target_config: TargetConfig::from_file(dummy_target_path)
                    .expect("Failed to load dummy target config"),
                unit_name: format!("{}_unit", id),
                unit_version: "0.0.1".to_string(),
                tags: Vec::new(),
                references: Vec::<TemplateReference>::new(),
                custom: HashMap::new(),
            },
            steps,
        })
    }

    #[test]
    fn test_basic_variable_operations() {
        let mut manager = VariableManager::new();

        // 设置变量
        manager
            .set_variable("template1", "step1", "test_var", "test_value")
            .unwrap();

        // 获取变量
        assert_eq!(
            manager.get_variable("test_var", Some("template1"), Some("step1")),
            Some("test_value".to_string())
        );
    }

    #[test]
    fn test_namespaced_variables() {
        let mut manager = VariableManager::new();

        // 注册命名空间
        manager.register_namespace("ns1", "template1");

        // 设置变量
        manager
            .set_variable("template1", "GLOBAL", "var1", "value1")
            .unwrap();

        // 通过命名空间访问
        assert_eq!(
            manager.get_variable("var1", Some("ns1"), Some("GLOBAL")),
            Some("value1".to_string())
        );
    }

    #[test]
    fn test_variable_replacement() {
        let mut manager = VariableManager::new();
        // 构造内容块，包含待替换的变量
        let content_blocks = vec![
            ContentBlock::Text("${greeting} ${name}!".to_string()),
            ContentBlock::Text("{{ t1::GLOBAL::greeting }} {{ name }}!".to_string()),
            ContentBlock::Text("{ t1::GLOBAL::greeting } { name }!".to_string()),
        ];
        // 创建一个虚拟模板对象
        let template_obj =
            create_dummy_template("t1", "Test content", content_blocks.clone(), vec![]);
        // 注册模板
        manager
            .register_template(&template_obj, Some("t1"))
            .unwrap();

        // 设置变量
        manager
            .set_variable("t1", "GLOBAL", "greeting", "Hello")
            .unwrap();
        manager
            .set_variable("t1", "GLOBAL", "name", "Alice")
            .unwrap();

        // 测试替换
        let get_text = |cb: &ContentBlock| {
            if let ContentBlock::Text(text) = cb {
                text.clone()
            } else {
                panic!("ContentBlock is not Text variant");
            }
        };

        assert_eq!(
            manager.replace_variables(&get_text(&content_blocks[0]), Some("t1"), Some("GLOBAL")),
            "Hello Alice!"
        );

        assert_eq!(
            manager.replace_variables(&get_text(&content_blocks[1]), Some("t1"), Some("GLOBAL")),
            "Hello Alice!"
        );

        assert_eq!(
            manager.replace_variables(&get_text(&content_blocks[2]), Some("t1"), Some("GLOBAL")),
            "Hello Alice!"
        );
    }

    #[test]
    fn test_content_block_variable_replacement() {
        let mut manager = VariableManager::new();

        // 构造内容块
        let content_blocks = vec![
            ContentBlock::Text("Welcome, {{ user }}!".to_string()), // 欢迎, {{ user }}!
            ContentBlock::Text("Score: {{ score }}".to_string()),   // 分数: {{ score }}
        ];
        let template_obj =
            create_dummy_template("template2", "Test content", content_blocks.clone(), vec![]);
        manager
            .register_template(&template_obj, Some("template2"))
            .unwrap();

        manager
            .set_variable("template2", "GLOBAL", "user", "XiaoMing")
            .unwrap(); // 小明
        manager
            .set_variable("template2", "GLOBAL", "score", "99")
            .unwrap();

        // 检查内容块变量替换
        if let ContentBlock::Text(ref text) = content_blocks[0] {
            let replaced_text = manager.replace_variables(text, Some("template2"), None);
            assert_eq!(replaced_text, "Welcome, XiaoMing!"); // 欢迎, 小明!
        } else {
            panic!("ContentBlock is not Text variant");
        }

        if let ContentBlock::Text(ref text) = content_blocks[1] {
            let replaced_code = manager.replace_variables(text, Some("template2"), None);
            assert_eq!(replaced_code, "Score: 99"); // 分数: 99
        } else {
            panic!("ContentBlock is not Text variant");
        }
    }

    #[test]
    fn test_content_block_conditional() {
        let mut manager = VariableManager::new();

        let content_blocks = vec![
            ContentBlock::Text("{{ passed == \"yes\" ? \"Passed\" : \"Failed\" }}".to_string()), // {{ passed == \"yes\" ? \"通过\" : \"未通过\" }}
            ContentBlock::Text("{{ score >= 60 ? \"Pass\" : \"Fail\" }}".to_string()), // {{ score >= 60 ? \"及格\" : \"不及格\" }}
        ];

        let template_obj =
            create_dummy_template("template3", "Test content", content_blocks.clone(), vec![]);
        manager
            .register_template(&template_obj, Some("template3"))
            .unwrap();

        // 设置变量
        manager
            .set_variable("template3", "GLOBAL", "passed", "no")
            .unwrap();
        manager
            .set_variable("template3", "GLOBAL", "score", "59")
            .unwrap();

        if let ContentBlock::Text(ref text) = content_blocks[1] {
            let replaced_score = manager.replace_variables(text, Some("template3"), None);
            assert_eq!(replaced_score, "Fail"); // 不及格
        } else {
            panic!("ContentBlock is not Text variant");
        }

        // 修改分数再测一次
        manager
            .set_variable("template3", "GLOBAL", "score", "60")
            .unwrap();
        if let ContentBlock::Text(ref text) = content_blocks[1] {
            let replaced_score2 = manager.replace_variables(text, Some("template3"), None);
            assert_eq!(replaced_score2, "Pass"); // 及格
        } else {
            panic!("ContentBlock is not Text variant");
        }
    }

    #[test]
    fn test_content_block_nested_variables() {
        let mut manager = VariableManager::new();

        let content_blocks = vec![
            ContentBlock::Text("Hello, {{ user }}! Status: {{ status }}".to_string()), // 你好, {{ user }}! 状态: {{ status }}
            ContentBlock::Text(
                "{{ status == \"active\" ? \"Welcome back\" : \"Please activate\" }}".to_string(),
            ), // {{ status == \"active\" ? \"欢迎回来\" : \"请激活\" }}
        ];

        let template_obj =
            create_dummy_template("template4", "Test content", content_blocks.clone(), vec![]);
        manager
            .register_template(&template_obj, Some("template4"))
            .unwrap();

        manager
            .set_variable("template4", "GLOBAL", "user", "ZhangSan")
            .unwrap(); // 张三
        manager
            .set_variable("template4", "GLOBAL", "status", "active")
            .unwrap();

        if let ContentBlock::Text(ref text) = content_blocks[0] {
            let replaced_text = manager.replace_variables(text, Some("template4"), None);
            assert_eq!(replaced_text, "Hello, ZhangSan! Status: active"); // 你好, 张三! 状态: active
        } else {
            panic!("ContentBlock is not Text variant");
        }

        if let ContentBlock::Text(ref text) = content_blocks[1] {
            let replaced_cond2 = manager.replace_variables(text, Some("template4"), None);
            assert_eq!(replaced_cond2, "Welcome back"); // 欢迎回来
        } else {
            panic!("ContentBlock is not Text variant");
        }
    }
}
