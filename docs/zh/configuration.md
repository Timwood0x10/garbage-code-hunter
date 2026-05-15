# 配置说明

## 项目配置文件

在项目根目录创建 `.garbage-code-hunter.toml`。工具会从目标目录向上搜索。

### 示例

```toml
[project-type]
web-service = {}

[whitelists]
# 不算魔法数字的数字
magic-numbers = [8080, 443, 80, 3000, 5000]

# 可接受的变量名
variable-names = ["ctx", "req", "res", "err", "db", "wg", "mu"]

# 降低敏感度的目录（仍会分析，但权重更低）
directories = ["vendor/", "generated/", "testdata/"]

# 完全排除的目录或文件模式
exclude-patterns = [
    "*.pb.go",
    "*.pulsar.go",
    "*_grpc.pb.go",
    "*.gen.ts",
    "*.generated.*",
    "node_modules/",
    "venv/",
    "__pycache__/",
    "vendor/",
]

[rules.naming]
enabled = true
severity = "mild"
allowed-names = ["id", "ok", "tx", "rx", "fs"]

[rules.unwrap]
enabled = true
threshold = 1
nuclear-threshold = 15

[rules.magic-number]
enabled = true
allowed-numbers = [200, 201, 204, 400, 401, 403, 404, 500]

[rules.println]
enabled = true

[[overrides]]
pattern = "generated/"
# 生成代码目录的覆盖设置
```

## 严重度等级

| 等级 | 权重 | 说明 |
|------|------|------|
| `mild` | 0.5 | 小问题，可以忽略 |
| `spicy` | 1.5 | 应该修复 |
| `nuclear` | 3.0 | 立即修复 |

## 命令行选项

```bash
# 基本用法
garbage-code-hunter analyze [PATH]

# 语言过滤
garbage-code-hunter analyze --lang go

# 排除模式
garbage-code-hunter analyze --exclude "vendor/*" --exclude "*.pb.go"

# LLM 模式（需要 API key）
garbage-code-hunter analyze --mode llm

# 输出格式
garbage-code-hunter analyze --format json
garbage-code-hunter analyze --format markdown
```
