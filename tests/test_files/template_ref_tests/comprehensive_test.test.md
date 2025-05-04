---
title: "综合系统测试"
target_config: "../../../targets/local/config.toml"
unit_name: "综合评估"
tags: ["comprehensive", "final"]
references:
  - template: "base_test.test.md"
    as: "base"
  - template: "performance_test.test.md"
    as: "perf"
  - template: "security_test.test.md"
    as: "sec"
---

# 综合系统测试

这个测试模板汇总了所有之前测试的结果，并给出最终评估。

## 系统状态汇总 {id="system-summary"}

```bash {id="gather-summary" exec=true}
echo "生成综合系统报告..."
echo "测试执行时间: {{ base.time }}"
echo "测试执行用户: {{ base.username }}"
echo "系统内核版本: {{ base.kernel }}"
echo "系统当前负载: {{ perf.system_load }}"
echo "----------------"
echo "性能测试分数: {{ perf.perf_score }}"
echo "安全权限分数: {{ sec.permission_score }}"
echo "平衡评估分数: {{ sec.balance_score }}"
echo "安全综合评级: {{ sec.security_rating }}"
```

## 最终系统评估 {id="final-evaluation"}

```bash {id="evaluate-final" exec=true extract.final_score=/Final system score:\s+(\d+)/}
echo "计算最终系统评分..."
PERF_WEIGHT=0.4
SEC_WEIGHT=0.6

# 计算加权最终得分
FINAL_SCORE=$(awk "BEGIN {print int({{ perf.perf_score }} * $PERF_WEIGHT + {{ sec.balance_score }} * $SEC_WEIGHT)}")
echo "Final system score: $FINAL_SCORE"
```

## 系统建议生成 {id="recommendations"}

```bash {id="generate-recommendations" exec=true extract.recommendation=/Recommendation:\s+(.+)/}
echo "生成系统建议..."
FINAL=$FINAL_SCORE

if [ $FINAL -ge 90 ]; then
  echo "Recommendation: 系统表现优秀，可以投入生产环境"
elif [ $FINAL -ge 80 ]; then
  echo "Recommendation: 系统表现良好，建议小幅优化后再投入生产"
elif [ $FINAL -ge 70 ]; then
  echo "Recommendation: 系统表现一般，需要进行性能和安全优化"
else
  echo "Recommendation: 系统表现不佳，需要重大改进后才能投入使用"
fi
```

# 综合测试报告

## 基础信息
- 测试时间: {{ base.time }}
- 执行用户: {{ base.username }}
- 系统内核: {{ base.kernel }}
- 系统负载: {{ perf.system_load }}

## 测试分数汇总
- 基础测试分数: {{ base.base_score }}
- 性能测试分数: {{ perf.perf_score }}
- 安全权限分数: {{ sec.permission_score }}
- 安全平衡分数: {{ sec.balance_score }}
- 安全综合评级: {{ sec.security_rating }}
- 最终系统分数: {{ final_score }}

## 系统建议
{{ recommendation }}

## 测试路径
本次测试执行了以下测试模板:
1. 基础测试 (base_test.test.md)
2. 性能测试 (performance_test.test.md) - 依赖基础测试
3. 安全测试 (security_test.test.md) - 依赖基础测试和性能测试
4. 综合测试 (comprehensive_test.test.md) - 依赖所有之前的测试