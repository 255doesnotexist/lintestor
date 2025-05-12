---
title: "基础测试"
target_config: "../../../targets/local/config.toml"
unit_name: "基础模块"
tags: ["base", "foundation"]
---

# 基础测试

这个测试模板定义了其他测试可能需要的基础变量。

## 系统信息收集 {id="system-info"}

```bash {id="collect-system" exec=true extract.kernel=/Linux\s+(\S+)/}
uname -a
```

```output {ref="collect-system"}
output
```

## 时间戳生成 {id="timestamp"}

```bash {id="get-timestamp" exec=true extract.time=/Current time:\s+(.+)/}
echo "Current time: $(date '+%Y-%m-%d %H:%M:%S')"
```

```output {ref="get-timestamp"}
output
```

## 用户信息获取 {id="user-info"}

```bash {id="get-user" exec=true extract.username=/Current user:\s+(.+)/}
echo "Current user: $(whoami)"
```

```output {ref="get-user"}
output
```

## 计算基准数值 {id="benchmarks"}

```bash {id="calc-base-values" exec=true extract.base_score=/Base score:\s+(\d+)/}
echo "Performing base calculations..."
sleep 1
echo "Base score: 85"
```

```output {ref="benchmarks"}
output
```

系统信息: 内核版本 {{ kernel }}
当前时间: {{ time }}
当前用户: {{ username }}
基准分数: {{ base_score }}