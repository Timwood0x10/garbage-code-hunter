# 工具指南

这个项目提供了一整套吐槽风格工具。大多数命令都支持 `-f/--format`，在需要本地化文案时也支持 `--lang`。

## `analyze`

核心源码分析命令。

```bash
garbage-code-hunter analyze
```

常用参数：
- `--harsh`
- `--verbose`
- `--top`
- `--issues`
- `--summary`
- `--brief`
- `--markdown`
- `--lang`
- `--exclude`
- `--educational`
- `--project-config`
- `--hall-of-shame`
- `--suggestions`
- `--format`
- `--llm`
- `--llm-provider`
- `--llm-endpoint`
- `--llm-model`
- `--llm-api-key`
- `--llm-timeout`
- `--config`

## `commit-roaster` / `cr`

分析 git 历史，吐槽弱鸡 commit 信息。

```bash
garbage-code-hunter commit-roaster --limit 50
```

参数：`--limit`、`--author`、`--since`、`--until`、`--branch`、`--format`

## `deps-shamer` / `ds`

检查依赖文件里是否存在高风险或混乱的依赖选择。

参数：`--format`

## `pr-title-hunter` / `pr`

从本地数据或 GitHub 获取 PR 标题并吐槽。

参数：`--limit`、`--repo`、`--state`、`--token`、`--author`、`--format`

## `scan`

运行主工具合集并合并结果。

参数：`--lang`、`--format`、`--save`、`--project-config`

## `badge`

生成 SVG 徽章。

参数：`--output`、`--style`、`--score`、`PATH`

## `trend`

查看分数历史。

参数：`--last`、`--format`

## `last-words` / `lw`

查找陈旧的 TODO/FIXME/HACK 注释。

参数：`--age`、`--format`、`--lang`

## `debt-invoice` / `debt`

估算技术债成本。

参数：`--format`、`--lang`

## `personality`

根据代码模式推断开发者画像。

参数：`--format`、`--lang`

## `decay`

分析 git 历史中的质量衰减。

参数：`--format`、`--lang`

## `autopsy`

输出根因风格的质量报告。

参数：`--format`、`--lang`

## `radar`

生成气味雷达图或终端视图。

参数：`--format`、`--lang`、`--output`

## `ci-bot`

输出 CI 风格审查内容。

参数：`--format`、`--lang`

## `persona`

按指定人格模式吐槽代码。

参数：`--persona`、`--format`、`--lang`

常见人格包括 `linux-kernel`、`silicon-valley`、`japanese-enterprise` 和 `rust-fanatic`。

## `danger-zone` / `dz`

突出显示最危险的文件。

参数：`--format`、`--lang`

## `team-roast`

按贡献者汇总问题。

参数：`--limit`、`--format`、`--lang`
