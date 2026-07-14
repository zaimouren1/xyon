// ci-guardian —— xyon 的第一位 AI 员工。
//
// 每次唤醒（`bun run guard`）做一件事：检查 CI。
//   绿 → 记录一次签名心跳，结束。
//   红 → 以 ≤20 轮的预算调查，产出分支修复或诊断上报；
//        每次工具调用都写入 xyon 账本，最后 seal 成证据包。
//
// 复利记忆：每次调查结束后，员工把学到的教训追加进 memory.md，
// 下次唤醒时作为上下文注入。第八周的裁定依据即在于此（见 CHARTER.md）。

import { query } from "@anthropic-ai/claude-agent-sdk";
import { appendFileSync, existsSync, mkdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
import {
  ensureIdentity,
  gh,
  record,
  xyon,
  GUARDIAN_HOME,
  MAX_TURNS,
  MEMORY_PATH,
  REPO,
  REPO_ROOT,
  REPORTS_DIR,
} from "./contract.ts";

interface CiRun {
  status: string;
  conclusion: string;
  headSha: string;
  url: string;
  displayTitle: string;
}

function latestCiRun(): CiRun {
  const raw = gh([
    "run", "list", "--repo", REPO, "--branch", "main", "--workflow", "ci",
    "--limit", "1", "--json", "status,conclusion,headSha,url,displayTitle",
  ]);
  const runs = JSON.parse(raw) as CiRun[];
  if (runs.length === 0) throw new Error("no CI runs found");
  return runs[0];
}

function loadMemory(): string {
  return existsSync(MEMORY_PATH) ? readFileSync(MEMORY_PATH, "utf8") : "（暂无记忆 —— 这是第一次调查）";
}

async function investigate(run: CiRun): Promise<void> {
  // 员工在 Bash 里跑 gh/git 也需要可用代理（本机默认代理已失效，见 memory）。
  process.env.https_proxy ??= "http://127.0.0.1:7897";
  process.env.http_proxy ??= "http://127.0.0.1:7897";
  const memory = loadMemory();
  let turns = 0;

  const result = query({
    prompt: [
      `你是 ci-guardian，xyon 仓库的 AI 员工。委任状：${REPO_ROOT}/CHARTER.md。`,
      `main 分支 CI 失败：${run.url}（commit ${run.headSha.slice(0, 8)}，"${run.displayTitle}"）。`,
      ``,
      `## 你此前积累的经验（来自过往调查）`,
      memory,
      ``,
      `## 任务`,
      `1. 用 \`gh run view\` 和 \`gh api\` 查看失败日志，定位根因。`,
      `2. 能修：在 fix/ci-<短描述> 分支上提交修复并 push（不得动 main），开 PR。`,
      `3. 不能修：写出具体诊断（根因、涉及文件、建议方案），不要抛出笼统问题。`,
      `4. 最后一步（必须执行）：把这次学到的可复用教训追加到 ${MEMORY_PATH}，`,
      `   格式：\`- [YYYY-MM-DD] 教训一句话（根因 → 对策）\`。已有条目不要改动。`,
      ``,
      `约束：预算 ${MAX_TURNS} 轮；secrets 一律不碰；仓库之外的系统一律不碰。`,
    ].join("\n"),
    options: {
      cwd: REPO_ROOT,
      maxTurns: MAX_TURNS,
      allowedTools: ["Bash", "Read", "Edit", "Write", "Grep", "Glob"],
      hooks: {
        PreToolUse: [{
          hooks: [async (input) => {
            turns++;
            record("tool_call", {
              tool: input.tool_name,
              input: summarize(input.tool_input),
              turn: turns,
            });
            return { continue: true };
          }],
        }],
      },
    },
  });

  let outcome = "";
  for await (const message of result) {
    if (message.type === "result") {
      outcome = message.subtype; // "success" | "error_max_turns" | ...
      record("investigation_end", {
        outcome,
        turns_used: turns,
        cost_usd: message.total_cost_usd ?? null,
        duration_ms: message.duration_ms ?? null,
      });
    }
  }

  // 封存本次调查的证据包
  mkdirSync(REPORTS_DIR, { recursive: true });
  const stamp = new Date().toISOString().slice(0, 19).replace(/[:T]/g, "-");
  const bundlePath = join(REPORTS_DIR, `investigation-${stamp}.json`);
  console.log(xyon(["seal", bundlePath]));
  console.log(`调查结束（${outcome}，${turns} 次工具调用）· 证据包：${bundlePath}`);
}

/** 工具输入摘要：只记形状与关键字段，避免把大文件内容塞进账本。 */
function summarize(input: unknown): unknown {
  const s = JSON.stringify(input) ?? "";
  return s.length <= 500 ? input : { truncated: s.slice(0, 500) };
}

async function main(): Promise<void> {
  ensureIdentity();
  const run = latestCiRun();
  const green = run.status === "completed" && run.conclusion === "success";

  record("ci_check", {
    sha: run.headSha,
    status: run.status,
    conclusion: run.conclusion,
    verdict: green ? "green" : "red",
  });

  if (green) {
    console.log(`CI 绿（${run.headSha.slice(0, 8)}）· 心跳已记录，保持沉默。`);
    return;
  }
  console.log(`CI 红（${run.headSha.slice(0, 8)}）· 开始调查，预算 ${MAX_TURNS} 轮。`);
  await investigate(run);
}

main().catch((err) => {
  // 员工自身故障也必须留痕，不允许静默死亡。
  try {
    record("guardian_error", { error: String(err) });
  } catch { /* 账本不可用时至少保留 stderr */ }
  console.error("ci-guardian 故障:", err);
  process.exit(1);
});
