// ci-guardian 的雇佣合同参数与共享工具函数。
// 委任状见 ../CHARTER.md —— 本文件里的数字必须与委任状一致。

import { execFileSync } from "node:child_process";
import { existsSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

export const REPO = "zaimouren1/xyon";
export const MAX_TURNS = 20; // 预算硬上限：每次唤醒最多 20 轮（雇主设定，2026-07-15）

const employeeDir = dirname(fileURLToPath(import.meta.url));
export const REPO_ROOT = join(employeeDir, "..");
export const GUARDIAN_HOME = join(REPO_ROOT, ".guardian"); // 员工的 XYON_HOME，含私钥，永不入库
export const MEMORY_PATH = join(employeeDir, "memory.md");
export const REPORTS_DIR = join(REPO_ROOT, "reports");

const XYON_BIN =
  process.env.XYON_BIN ??
  join(REPO_ROOT, "target", "release", process.platform === "win32" ? "xyon.exe" : "xyon");

// gh CLI 走可用代理（本机 7897，见 memory）；允许环境覆盖。
export const PROXY_ENV = {
  ...process.env,
  https_proxy: process.env.https_proxy ?? "http://127.0.0.1:7897",
};

export function xyon(args: string[]): string {
  return execFileSync(XYON_BIN, ["--home", GUARDIAN_HOME, ...args], {
    encoding: "utf8",
  }).trim();
}

/** 确保员工身份存在（首次运行时自动 init）。 */
export function ensureIdentity(): void {
  if (!existsSync(join(GUARDIAN_HOME, "key"))) {
    console.log(xyon(["init"]));
  }
}

export function record(type: string, payload: unknown): void {
  xyon(["record", type, JSON.stringify(payload)]);
}

export function gh(args: string[]): string {
  return execFileSync("gh", args, { encoding: "utf8", env: PROXY_ENV }).trim();
}
