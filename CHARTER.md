# 委任状 —— xyon 的第一位 AI 员工

*(English summary at the bottom · 英文摘要见文末)*

这份文件是本仓库的创始实验。它是一份**委任状**，不是计划书：
指定一个 Agent、一项职责、一笔预算、一条可证伪的验收标准，
然后运行八周。

## 任命

- **员工**：`ci-guardian` —— 基于 Claude Agent SDK 构建的 AI Agent。
- **雇主**：本仓库的人类所有者。
- **入职日**：第一份 `ci-guardian` 证据包提交进仓库的那一天。
- **任期**：八周。续约取决于文末的裁定条款。

## 职责

看护 `xyon` 仓库的 CI 健康。

- CI 绿：保持沉默。
- CI 红：先调查，然后要么提交修复（所有变更必须经人类审批），
  要么带着诊断结论升级上报。
- 每周：以证据包形式向 `reports/` 提交一份签名工作报告。

## 权限与边界

- **可以**：读取仓库、运行构建与测试、在分支上提议 commit。
- **不可以**：推送到 `main`、接触任何 secret、超出预算消费、
  在本仓库之外采取任何行动。
- 每次工具调用都记录进 xyon 账本；每个完成的任务都用 `xyon seal`
  封存，任何人都可用 `xyon verify` 验证。

## 预算

- API 成本：按月设上限（雇主在雇佣时设定）。
- 人类注意力：升级上报必须附带诊断结论，而不是抛出问题。

## 可证伪的裁定（第八周）

实验成功的唯一标准：第八周的员工在这项具体工作上**可度量地比
第一周更胜任**，且由它自己的签名证据证明——诊断更快、误报更少、
或报告中引用了此前几周学到的教训。

如果记忆没有复利，那么本项目的核心论题——Agent 可以积累值得信任的
可验证履历——就以最低的成本、在八周内被证伪。届时本文件将如实记录
失败，而不是被删除。

## 状态

- [x] 员工已于 2026-07-15 雇佣。
  - 身份（principal）：`DtsE96dhbXH1hUbpGRjcWt8jKerDwooSSJ37PT7Up4mS`
  - 公钥：`baf03b57ea99473c91004c239669beffe903ec45cc8c256c5732fdf0f3f2d434`
  - 预算：每次唤醒 ≤ 20 轮工具调用；唤醒方式：手动/本地定时（`bun run guard`）
  - 首份证据包：[`reports/hired-2026-07-15.json`](reports/hired-2026-07-15.json)
  - 八周任期起算日：2026-07-15；裁定日：2026-09-09

---

## English Summary

This is a mandate, not a plan: one AI employee (`ci-guardian`, built on
the Claude Agent SDK), one responsibility (keep xyon's CI healthy), one
budget, one falsifiable verdict. The experiment succeeds only if the
week-8 employee is measurably more competent than the week-1 employee,
proven by its own signed evidence bundles in `reports/`. If memory does
not compound, the thesis fails cheaply in eight weeks — and this file
will record the failure rather than be deleted.
