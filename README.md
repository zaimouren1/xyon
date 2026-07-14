# xyon

为 AI Agent 的行为提供签名的、防篡改的证据。

*(English summary at the bottom · 英文摘要见文末)*

xyon 是一个很小的单一二进制程序：它把 Agent 做过的每件事记录进一份
**只追加、哈希链接、ed25519 签名**的账本，并封存为一个证据包——
**任何人都可以离线验证**，不需要账号、不需要联网、也不需要信任递交证据的人。

## 当前状态：v0.0.1 —— 诚实且微小

今天已经存在的（以下每一条都已实现并有测试覆盖）：

- `xyon init` —— 创建本地 ed25519 身份
- `xyon record <type> [json]` —— 向账本追加一条签名事件
- `xyon log` —— 打印账本内容
- `xyon seal <out.json>` —— 将账本封存为自包含的证据包
- `xyon verify <bundle.json>` —— 独立验证哈希链与全部签名

尚不存在的：Agent 集成（Claude Code hooks）、审批闸门、回滚、
[CHARTER.md](CHARTER.md) 中描述的 `ci-guardian` 员工。
**本 README 永远只描述真实存在的东西。**

## 安装

目前没有预构建的发布版本，需要从源码构建：

```bash
git clone https://github.com/zaimouren1/xyon
cd xyon
cargo build --release   # 需要 Rust 1.75+
```

## 60 秒上手

```bash
export XYON_HOME=$(mktemp -d)   # 隔离演示环境
xyon init
xyon record task_start '{"mission":"demo"}'
xyon record tool_call  '{"tool":"bash","cmd":"cargo test"}'
xyon record task_end   '{"result":"pass"}'
xyon seal evidence.json
xyon verify evidence.json
# ✓ signature valid · chain intact
```

现在试着篡改 `evidence.json` 里任何事件的任何一个字节，再运行
`xyon verify`——验证失败。删掉一条事件——失败。调换顺序——失败。
**这就是全部意义所在。**

## 为什么做这个

Agent 能力的增长速度远快于它获得信任的速度。缺的那块拼图不是智能，
而是**证据**：一份 Agent（或它的厂商、它的操作者）无法悄悄改写的
行为记录。xyon 是这份记录的最小可行形态——先把它造出来，之后的一切
才有立足之地。

更长期的论题写在 [CHARTER.md](CHARTER.md)：拥有可验证履历的 Agent
——是员工，而不是工具。

## 设计

- 一份账本 = 一个 JSONL 文件。每行一条事件，签名覆盖其规范化字节，
  并携带前一条事件的哈希。
- 一个证据包 = 账本全文 + 对链头的封存签名。
- 验证器只需要证据包本身。私钥永远不离开 `$XYON_HOME`。

## 许可证

Apache-2.0

---

## English Summary

xyon is a single small binary that records what an AI agent did into an
append-only, hash-chained, ed25519-signed ledger, and seals it into an
evidence bundle that anyone can verify offline — no account, no network,
no trust in the person handing it over.

v0.0.1 ships five commands (`init`, `record`, `log`, `seal`, `verify`),
all implemented and tested. Agent integration, approval gates, and the
`ci-guardian` experiment described in [CHARTER.md](CHARTER.md) do not
exist yet. This README only ever describes what is real.
