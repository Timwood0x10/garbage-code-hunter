# 配置说明

Garbage Code Hunter 会从 `.garbage-code-hunter.toml` 读取项目级配置。
程序会从目标路径开始向上查找该文件。

## 配置示例

```toml
[project-type]
web-service = {}

[whitelists]
magic-numbers = [8080, 443, 80, 3000, 5000]
variable-names = ["ctx", "req", "res", "err", "db", "wg", "mu"]
directories = ["vendor/", "generated/", "testdata/"]
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
```

## 配置分区

- `project-type`：分析上下文的项目类型提示
- `whitelists.magic-numbers`：不应该被报出的数字
- `whitelists.variable-names`：允许的变量名
- `whitelists.directories`：降低敏感度的目录
- `whitelists.exclude-patterns`：直接跳过的文件或路径
- `rules.*`：各规则开关和阈值
- `signals.*`：特殊信号检测的配置
- `overrides`：针对生成目录或特殊路径的覆盖配置

## 严重度等级

| 等级 | 权重 | 含义 |
|---|---:|---|
| `mild` | 0.5 | 轻微问题 |
| `spicy` | 1.5 | 建议修复 |
| `nuclear` | 3.0 | 立即修复 |

## 命令行参数

```bash
# 分析路径
garbage-code-hunter analyze [PATH]

# 按语言过滤
garbage-code-hunter analyze --lang go

# 排除路径
garbage-code-hunter analyze --exclude "vendor/*" --exclude "*.pb.go"

# 保存扫描结果
garbage-code-hunter scan --save

# 输出格式
garbage-code-hunter analyze --format json
garbage-code-hunter analyze --format markdown
```

## LLM 模式

程序支持切换到 LLM 生成吐槽文案的模式。
使用 `--llm`，并结合 `--llm-provider`、`--llm-endpoint`、`--llm-model`、`--llm-api-key`、`--llm-timeout`。
当前支持的 provider 包括 `ollama` 和通用的 OpenAI 风格接口。

示例：

```bash
garbage-code-hunter analyze --llm --llm-provider ollama
```
