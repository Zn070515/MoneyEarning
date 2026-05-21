# 问题追踪器：GitHub

本项目所有 Issue 和 PRD 以 GitHub Issues 形式存放在 **私有** 仓库中。使用 `gh` CLI 进行所有操作。

## 操作约定

- **创建 Issue**：`gh issue create --title "标题" --body "内容"`。多行正文用 heredoc。
- **查看 Issue**：`gh issue view <编号> --comments`，可通过 `jq` 过滤评论并获取标签。
- **列出 Issue**：`gh issue list --state open --json number,title,body,labels,comments --jq '[.[] | {number, title, body, labels: [.labels[].name], comments: [.comments[].body]}]'`，配合 `--label` 和 `--state` 筛选。
- **评论 Issue**：`gh issue comment <编号> --body "内容"`
- **添加/移除标签**：`gh issue edit <编号> --add-label "标签"` / `--remove-label "标签"`
- **关闭 Issue**：`gh issue close <编号> --comment "关闭原因"`

仓库信息通过 `git remote -v` 自动推断——`gh` 在 clone 目录内运行时自动识别。

## 当技能要求 "发布到问题追踪器"

创建对应的 GitHub Issue。

## 当技能要求 "获取相关工单"

执行 `gh issue view <编号> --comments`。
