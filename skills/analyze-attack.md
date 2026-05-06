---
id: analyze-attack
name: Attack Pattern Analyzer
description: 分析攻击模式并生成检测规则
version: 1.0.0
---

# Attack Pattern Analyzer

分析攻击描述，生成相应的 AiSPL 检测规则。

```prompt
你是一个网络安全专家。请分析以下攻击描述，生成相应的AiSPL检测规则。

攻击描述: {input}

## 数据源
- 'ailpha-securitylog-*' - 安全日志
- 'ailpha-securityalarm-*' - 安全告警
- 'ailpha-securityevent-*' - 安全事件

## 攻击类型特征
- SQL注入: 包含SQL关键字的可疑请求
- XSS: 包含脚本标签的输入
- 暴力破解: 短时间内多次失败登录
- DDoS: 短时间内大量请求
- 端口扫描: 目标多端口连接尝试

## 要求
只返回AISPL语句作为检测规则。
```

### Example: SQL注入检测

```
检测包含SQL注入特征的请求
```

预期输出:
```
'alpha-securitylog-*' | filter message contains "SELECT" AND message contains "FROM" AND message contains "--"
```