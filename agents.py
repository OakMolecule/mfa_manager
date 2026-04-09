#!/usr/bin/env python3
"""
MFA Manager Multi-Agent System
Agents: Architect → Reviewer → Coder → Tester
"""

import anthropic
import sys
from typing import Optional

client = anthropic.Anthropic()
MODEL = "claude-opus-4-6"

# ── Terminal colors ──────────────────────────────────────────────────────────
RESET  = "\033[0m"
BOLD   = "\033[1m"
DIM    = "\033[2m"

COLORS = {
    "architect": "\033[36m",   # Cyan
    "reviewer":  "\033[33m",   # Yellow
    "coder":     "\033[32m",   # Green
    "tester":    "\033[35m",   # Magenta
    "thinking":  "\033[90m",   # Dark gray
    "header":    "\033[34m",   # Blue
}

def banner(role: str, title: str) -> None:
    c = COLORS.get(role, "")
    width = 70
    print(f"\n{c}{BOLD}{'━' * width}{RESET}")
    print(f"{c}{BOLD}  {title}{RESET}")
    print(f"{c}{BOLD}{'━' * width}{RESET}\n")

def print_thinking(text: str) -> None:
    c = COLORS["thinking"]
    for line in text.strip().splitlines():
        print(f"{c}{DIM}  💭 {line}{RESET}")

def print_output(role: str, text: str) -> None:
    c = COLORS.get(role, "")
    for line in text.splitlines():
        print(f"{c}{line}{RESET}")

# ── Core: stream one agent turn ──────────────────────────────────────────────
def run_agent(
    role: str,
    system: str,
    user_message: str,
    show_thinking: bool = True,
) -> str:
    """Stream a single agent turn; return the full assistant text."""
    full_text = ""
    full_thinking = ""

    with client.messages.stream(
        model=MODEL,
        max_tokens=16000,
        thinking={"type": "adaptive"},
        system=system,
        messages=[{"role": "user", "content": user_message}],
    ) as stream:
        for event in stream:
            # Thinking delta
            if (
                event.type == "content_block_delta"
                and event.delta.type == "thinking_delta"
            ):
                full_thinking += event.delta.thinking
                if show_thinking:
                    print_thinking(event.delta.thinking)

            # Text delta
            elif (
                event.type == "content_block_delta"
                and event.delta.type == "text_delta"
            ):
                full_text += event.delta.text
                print_output(role, event.delta.text)
                sys.stdout.flush()

    return full_text


# ── Agent system prompts ─────────────────────────────────────────────────────
ARCHITECT_SYSTEM = """你是一位资深 Rust 系统架构师，专精密码学与安全应用。
你的任务是为 MFA Manager 设计核心 TOTP 引擎的 Rust 架构。

输出格式：
1. 模块结构（mod 树形图）
2. 核心数据结构（struct/enum，含字段与生命周期注解）
3. 关键 trait 设计（含方法签名）
4. 依赖清单（Cargo.toml dependencies，含版本与用途说明）
5. 安全考量（密钥存储、内存清零、错误处理策略）
6. 模块间数据流图（ASCII art）

要求：遵循 RFC 6238 (TOTP) / RFC 4226 (HOTP)，密钥加密使用 AES-256-GCM，
密钥派生使用 Argon2id，内存中的密钥材料必须在 Drop 时归零。"""

REVIEWER_SYSTEM = """你是一位严格的 Rust 代码评审专家，专注于安全性与正确性。
你收到一份架构设计文档，需要对其进行深度评审。

评审维度：
- 🔐 安全性：密钥管理、内存安全、侧信道防护
- 🦀 Rust 惯例：所有权、生命周期、错误处理（? 运算符、thiserror/anyhow）
- 📐 架构合理性：模块职责单一、接口最小化
- ⚡ 性能：不必要的 clone/copy、锁竞争风险
- 🧪 可测试性：mock 友好、依赖注入

输出格式：
P0（阻塞）、P1（必须改）、P2（建议）分级，每条给出具体修改方案。
最后给出「评审结论」：通过 / 条件通过 / 打回重做。"""

CODER_SYSTEM = """你是一位专精密码学的 Rust 程序员。
你收到经过评审的架构设计，需要实现核心 TOTP/HOTP 引擎。

实现要求：
- 完整的 Rust 代码，可直接放入项目（lib.rs + 子模块）
- 所有 pub API 必须有文档注释（/// ）
- 使用 zeroize crate 对密钥材料归零
- 错误类型使用 thiserror 定义
- 不使用 unwrap()，所有错误显式处理
- TOTP 窗口容忍：±1 步长（防时钟漂移）

输出结构：
```
// === src/lib.rs ===
<代码>

// === src/totp.rs ===
<代码>

// === src/crypto.rs ===
<代码>

// === src/error.rs ===
<代码>
```"""

TESTER_SYSTEM = """你是一位 Rust 测试工程师，擅长密码学协议的测试。
你收到实现代码，需要编写完整的测试套件。

测试类型：
1. 单元测试（#[test]）：每个函数的正常路径
2. RFC 测试向量：使用 RFC 4226 附录 D 的官方 HOTP 测试向量
3. 属性测试（proptest）：随机输入的不变量验证
4. 安全测试：
   - 密钥归零验证（内存模式检查）
   - 时间常数比较（防时序攻击）
   - 无效输入拒绝
5. 集成测试（tests/integration_test.rs）

输出完整的测试代码，包含 Cargo.toml 中需要添加的 dev-dependencies。
每个测试都要有注释说明测试意图。"""


# ── Orchestrator ─────────────────────────────────────────────────────────────
def main() -> None:
    print(f"\n{COLORS['header']}{BOLD}")
    print("╔══════════════════════════════════════════════════════════════════╗")
    print("║          MFA Manager — Multi-Agent Engineering System           ║")
    print("║                 claude-opus-4-6 · Adaptive Thinking             ║")
    print("╚══════════════════════════════════════════════════════════════════╝")
    print(RESET)
    print(f"{DIM}Task: Design and implement the core TOTP engine for MFA Manager in Rust{RESET}\n")

    # ── Phase 1: Architecture Design ─────────────────────────────────────────
    banner("architect", "🏛  Phase 1 / 4 — Architect Agent: Designing TOTP Engine")
    design_doc = run_agent(
        role="architect",
        system=ARCHITECT_SYSTEM,
        user_message=(
            "请为 MFA Manager 的核心 TOTP 引擎设计完整的 Rust 架构。\n"
            "项目目标：本地优先的桌面端 MFA 工具，支持 TOTP/HOTP，\n"
            "密钥加密存储（AES-256-GCM），主密码派生（Argon2id），跨平台（macOS/Linux/Windows）。"
        ),
        show_thinking=True,
    )

    # ── Phase 2: Design Review ────────────────────────────────────────────────
    banner("reviewer", "🔍  Phase 2 / 4 — Reviewer Agent: Architecture Review")
    review_doc = run_agent(
        role="reviewer",
        system=REVIEWER_SYSTEM,
        user_message=(
            f"请对以下 MFA Manager TOTP 引擎架构设计进行严格评审：\n\n"
            f"---\n{design_doc}\n---\n\n"
            "重点关注：Rust 内存安全、密钥生命周期管理、错误处理策略。"
        ),
        show_thinking=True,
    )

    # ── Phase 3: Implementation ───────────────────────────────────────────────
    banner("coder", "⚙️  Phase 3 / 4 — Coder Agent: Implementing TOTP Engine")
    code_doc = run_agent(
        role="coder",
        system=CODER_SYSTEM,
        user_message=(
            f"根据以下已评审的架构设计，实现 MFA Manager 的 TOTP 引擎：\n\n"
            f"【架构设计】\n{design_doc}\n\n"
            f"【评审意见】（请在实现中修复所有 P0/P1 问题）\n{review_doc}\n\n"
            "请输出完整、可编译的 Rust 代码。"
        ),
        show_thinking=True,
    )

    # ── Phase 4: Testing ──────────────────────────────────────────────────────
    banner("tester", "🧪  Phase 4 / 4 — Tester Agent: Writing Test Suite")
    test_doc = run_agent(
        role="tester",
        system=TESTER_SYSTEM,
        user_message=(
            f"为以下 MFA Manager TOTP 引擎实现编写完整的测试套件：\n\n"
            f"【实现代码】\n{code_doc}\n\n"
            "包含：RFC 4226 官方测试向量、属性测试、安全测试、集成测试。"
        ),
        show_thinking=True,
    )

    # ── Save artifacts ────────────────────────────────────────────────────────
    artifacts = {
        "design.md":  ("Architecture Design", design_doc),
        "review.md":  ("Review Report",       review_doc),
        "code.md":    ("Implementation",      code_doc),
        "tests.md":   ("Test Suite",          test_doc),
    }

    print(f"\n{COLORS['header']}{BOLD}{'━' * 70}{RESET}")
    print(f"{COLORS['header']}{BOLD}  Saving artifacts…{RESET}")
    for filename, (title, content) in artifacts.items():
        path = f"/home/oak/mfa_manager/{filename}"
        with open(path, "w", encoding="utf-8") as f:
            f.write(f"# {title}\n\n{content}\n")
        print(f"  ✓  {path}")

    print(f"\n{COLORS['header']}{BOLD}  All phases complete.{RESET}\n")


if __name__ == "__main__":
    main()
