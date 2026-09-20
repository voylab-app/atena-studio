# 🧠 01. Atena Memory System (Simple, Lightweight, and Efficient Architecture)

This document describes the architecture of the **Atena Memory System**. The core objective is to be **extremely lightweight, instantaneous (0ms), and resource-efficient**, running seamlessly on **any standard computer** (from 8 GB RAM laptops to dedicated workstations) without requiring heavy hardware or overloading the machine with background inference.

---

## 🎯 1. Design Principles

1. **Extreme Lightness (< 5 MB RAM):** Zero unnecessary VRAM consumption and zero background processing overhead.
2. **Instant O(1) Lookups:** Real-time context retrieval (< 1ms) prior to dispatching prompts to the language model.
3. **Dual Memory Layers (Declarative + Procedural):**
   - **Declarative Memory:** Stores facts, names, preferences, and rules.
   - **Procedural Memory:** Learns and refines **how to execute tasks and step-by-step methods**.
4. **No Heavy Background Inference:** The system avoids re-processing entire chat histories, eliminating freezes and Out-of-Memory (OOM) crashes.
5. **Unified, Robust Storage:** Instead of multiple brittle binary files, memory is structured in a direct, readable, and resilient format (`~/.atena/memoria.db` and `~/.atena/brain/prefrontal/skills.json`).
6. **Natural Learning and Refinement:** The AI acquires facts and skills directly during live dialogue, polishing routines with every piece of feedback.

---

## 🗂️ 2. Declarative Memory: Facts and Relations

Factual memory is structured into **Simple Associative Triads** (Subject ➔ Relation ➔ Value):

```text
┌──────────────┐         ┌──────────────────────┐         ┌───────────────────────────┐
│   Subject    │  ────►  │       Relation       │  ────►  │      Object / Value       │
└──────────────┘         └──────────────────────┘         └───────────────────────────┘
 "User"                    "has name"                       "Maria"
 "Maria"                   "is developer of"                "Rust and TypeScript"
 "Maria"                   "has partner"                    "Carlos Silva"
 "Passport"                "is stored in"                   "Office drawer"
 "Atena"                   "must avoid"                     "Using Python 2 (Use v3)"
 "Carlos Drummond"         "is known as"                    "Great Poet"
```

### 🏷️ Types of Declarative Memory:
- **Identities & Facts:** Names, family, occupation, preferences, and personal history.
- **Object Locations:** Where physical or digital items are kept.
- **Rules & Safeguards (Inhibitory):** Corrections supplied by the user (*"don't do that"*, *"always use X"*), preventing the AI from repeating prior mistakes.

---

## 🛠️ 3. Procedural Memory: Folder-Based Skills & Executable Scripts

Procedural memory stores the **"How-To"**. In Atena Studio, each procedural skill is stored in its own dedicated folder under `~/.atena/skills/<skill-slug>/`, supporting **executable scripts and shell commands** alongside step-by-step instructions.

### 📁 3.1 Anatomy of a Skill Directory
```text
~/.atena/skills/
├── git-workflow/
│   ├── skill.json          # Complete JSON specification, triggers, steps, env vars
│   ├── SKILL.md            # Human-readable markdown specification and instructions
│   └── scripts/            # Executable helper scripts (.sh, .py, .js, .bat, etc.)
│       ├── pre-check.sh
│       └── commit-push.sh
```

### 📋 3.2 Executable Steps & Skill Structure
```text
┌────────────────────────────────────────────────────────────────────────────┐
│ 🛠️ SKILL: Asset Purchase Planning (v1.2 Refined)                           │
│ ⚡ Triggers: "buy motorcycle", "plan purchase", "how long to buy"           │
│ 📁 Folder: ~/.atena/skills/asset-purchase-planning                         │
├────────────────────────────────────────────────────────────────────────────┤
│ 📋 Execution Step-by-Step:                                                 │
│   1. Analyze reported income and projections up to the target date         │
│   2. Deduct monthly fixed expenses before computing available balance      │
│   3. Run validation calculation [Script: scripts/calc_budget.py]           │
│   4. Deduct a 15% safety buffer for contingencies                          │
│   5. Calculate the exact number of months needed to reach the target sum   │
│   6. Present a clear summary with timeline and monthly savings target      │
├────────────────────────────────────────────────────────────────────────────┤
│ 📜 Refinement History:                                                     │
│   • v1.0: Initial creation of timeline calculation by accumulated income   │
│   • v1.1: Added mandatory deduction of fixed expenses                      │
│   • v1.2: Added 15% contingency safety margin and calculation script       │
└────────────────────────────────────────────────────────────────────────────┘
```

### 💻 3.3 Unified Command & Script Execution Engine (`run_command` & `run_skill_script`)
- **Global `run_command` Tool:** The canonical tool for executing CLI commands on behalf of the user is `run_command`. It serves both learned procedural skill steps and user-requested diagnostics, filesystem inspections, and routine automations. Legacy `run_skill_command` calls are seamlessly canonicalized to `run_command`.
- **Script File Execution (`run_skill_script`):** Executes scripts stored in the procedural skill's `scripts/` directory with scoped arguments.
- **Permission Modes (`ask` vs `auto` / Cron-ready):**
  - **`ask` (Default):** Interactive safety mode. Displays an in-chat card requiring user approval before running.
  - **`auto` ("Always Allow"):** Autonomous mode. Skills or commands marked as "Always Allow" execute without blocking user confirmation. This is foundational for upcoming background scheduled tasks (Cron) and frictionless daily routines.
  - **In-Chat & Modal Management:** Users can toggle permissions directly on the skill card or click **"Always Allow"** (`Zap` icon) directly within any pending tool authorization card in the chat.
- **Multi-Platform Dispatcher:** Automatically selects `/bin/zsh`, `/bin/sh`, `python3`, `node`, `cmd.exe`, or `powershell.exe` depending on script extension and OS.
- **Environment & CWD Context:** Injects `ATENA_SKILL_DIR` pointing to the skill folder, merges skill-specific environment variables, and defaults working directory safely.
- **Integrated Terminal Modal:** Provides an in-app dark terminal viewer displaying exit codes, execution duration (ms), real-time status chips, standard output (`stdout`), and standard error (`stderr`).

### 🛡️ 3.4 Destructive Command Safeguards & Rules
To protect system and user data, the execution engine enforces strict safety guardrails:
1. **Blocked Destructive Deletions:** High-risk recursive deletion patterns such as `rm -rf /`, `rm -rf ~`, `rm -rf *`, `rm -rf $HOME`, `rd /s /q c:\`, and `del /f /s /q *.*` are automatically blocked before spawning any subprocess.
2. **Blocked Disk & Partition Destructors:** Raw drive commands like `mkfs`, `format c:`, `dd if=/dev/zero`, `dd of=/dev/sd*`, and partition wipers are immediately rejected.
3. **Exploit & Malicious Pattern Prevention:** Fork bombs (`:(){ :|:& };:`) and blanket root permission changes (`chmod -R 777 /`) are blocked.
4. **AI Safety Prompt Directive:** The system prompt explicitly instructs the AI that destructive commands are forbidden and that non-destructive, scoped alternatives must always be chosen.

### 🔄 3.5 How the Skill Refinement Cycle Works:
1. **Teach & Synthesize:** The user explains a procedure (*"when I ask for X, do 1. ... 2. ... 3. ..."*) or asks the AI to automate tasks discussed in the conversation. The AI can invoke `create_procedural_skill` to register the new skill, complete with executable script files (`.sh`, `.py`, `.js`) created in `scripts/`.
2. **Execute:** Upon receiving a prompt with matching keywords (triggers), Atena loads the steps, folder scripts, and execution recipes.
3. **Refine & Edit:** When the user asks to modify, enhance, or correct an existing skill, the AI invokes `update_procedural_skill` (or `skills_update_with_scripts`), which **increments the skill version** (e.g., `v1` ➔ `v2`), updates steps, commands, triggers, or script files, and logs refinement notes.

### 🎛️ 3.6 Skill Lifecycle: Activation / Deactivation & Deletion Safeguards
1. **Individual Skill Activation Toggle (`enabled: bool`):**
   - Each procedural skill contains an `enabled` state (default `true`).
   - In the Memory UI, users can toggle individual skills between **Active** and **Disabled** directly from the card badge or within the skill refinement modal.
   - Disabled skills are excluded during trigger matching (`find_matching_skills`), ensuring they are never injected into the LLM system prompt context and will not be triggered automatically.
   - The status is persisted directly in `skill.json` and documented in `SKILL.md`.
2. **Safe Deletion with Confirmation Modal:**
   - Skill deletion permanently removes the skill directory and all script assets from `~/.atena/skills/<skill-slug>/`.
   - To avoid accidental loss of refined procedures, deleting a skill opens a dedicated confirmation dialog displaying the skill title, version, and folder location before any filesystem operation takes place.

---

## ⚡ 4. Real-Time Integrated Workflow (Step-by-Step)

### 4.1 During Live Conversation
1. **User sends a message:**
   - The system instantly queries relevant facts (declarative) and matching skills (procedural).
2. **Transparent Prompt Injection:**
   - If active facts or skills exist, only strictly relevant context is attached to the AI context in RAM O(1).
3. **Autonomous Execution and Recording:**
   - The AI responds adhering to the skill steps.
   - If new facts, procedures, or code automations are synthesized, the AI writes them immediately using transparent tags or tool calls:
     ```xml
     <memory subject="User" property="Name: Maria" valence="1" />
     <skill name="Git Audit" description="..." triggers="audit" steps="1. Run audit [cmd: git status]" />
     ```
     ```json
     <tool_call>
     {"name": "create_procedural_skill", "arguments": {"name": "Git Audit", "scripts": [{"filename": "audit.sh", "content": "#!/bin/bash\ngit status --short\n"}]}}
     </tool_call>
     ```
     Or to modify/update an existing skill:
     ```json
     <tool_call>
     {"name": "update_procedural_skill", "arguments": {"name": "Git Audit", "refinement_note": "Added untracked files filter", "steps": [{"order": 1, "instruction": "Check git status", "command": "git status -uall"}]}}
     </tool_call>
     ```

---

## 🎛️ 5. Granular Memory Controls (Independent Subsystems)

To allow end users full control over token consumption and contextual injection, Atena Studio decouples cognitive memory into three independently toggleable layers:

1. **Facts Network (`enable_facts_memory` - Declarative Layer):**
   - Enables associative knowledge graph queries, user profile extraction, and autonomous parsing of `<memory>` and `<forget>` XML tags.
   - When paused, factual associations are skipped during prompt building and no facts are auto-learned.
2. **Skills & Automations (`enable_skills_memory` - Procedural Layer):**
   - Injects learned step-by-step recipes and procedural skill execution instructions.
   - When paused, procedural skills (`create_procedural_skill`, `update_procedural_skill`, `run_skill_script`, `run_command`) are omitted from the inference context and suppressed. Core native tools (web search, webpage reader, task scratchpad, routine scheduler) remain independent and fully accessible.
3. **Logbook / Diary (`enable_episodic_memory` - Episodic Layer):**
   - Manages chronological turn continuity and persists user/assistant dialogue turns into the daily wakefulness buffer (`diario_vigilia.atena`) and linked episodic Markdown logs.
   - When paused, past turn episodes are not retrieved and new wakefulness events are suspended.
4. **Universal Channel & Gateway Governance:**
   - External communication gateways (Telegram, Discord) and autonomous background schedulers strictly inherit and honor these layer toggles in real time. Pausing or enabling memory layers in Atena Studio takes effect universally across all active communication channels.

### 🧭 Control Surfaces
- **Global Settings Panel (`SettingsScreen.vue`):** Under the Master Cognitive Memory switch, three dedicated toggle switches allow fine-tuning of each layer.
- **Direct In-Memory Controls (`MemoryScreen.vue`):** Each tab provides real-time status indicators (Active vs. Paused) and a direct toggle switch on the header bar, enabling users to pause or resume any layer without leaving the visualizer.

---

## ⚡ 6. Memory Optimization and Synchronization (Zero Hardware Cost)

- **Temporal & Usage Decay:** Temporary or inactive facts gradually lose relevance.
- **Deduplication and Pruning:** Executed in milliseconds via procedural code, without consuming GPU or VRAM.
- **In-App Control Panel:**
  - **"Fact Network"** Tab: Real-time visualization and management of the entity-relationship graph.
  - **"Test Association (Spreading Activation)"** Tool: Interactive testbench featuring live chips from your own memory graph.
  - **"Recent Facts Buffer"** Drawer: Captured dialogue snippets awaiting consolidation.
  - **"Synchronize"** Button: Consolidates facts and optimizes connections instantaneously (~0ms).
  - **"Skills & Tasks"** Tab: View, create, refine, and edit step-by-step execution recipes.
  - **"Captain's Log / Diary"** Tab: View chronological episodes and wakefulness turns.

---

## 📊 7. Comparison: Legacy Architecture vs. Modern Architecture

| Aspect | Legacy Architecture (Complex) | Modern Architecture (Simple & Efficient) |
| :--- | :--- | :--- |
| **Files** | 8+ binary files partitioned by brain lobes | Direct unified file |
| **Procedural Memory** | Non-existent (isolated facts only) | **Step-by-step skills with versioning and refinement** |
| **VRAM Consumption** | High (risk of Out-of-Memory) | **0 MB additional VRAM** |
| **RAM Consumption** | Variable with decompression spikes | **< 5 MB fixed** |
| **Query Latency** | Traversal of heavy graph structures | **Instantaneous (< 1ms)** |
| **Compatibility** | Required high-end hardware | **Runs smoothly on any basic PC or Mac** |
| **Refinement** | Not supported | **Continuous feedback-driven refinement (v1, v2...)** |

---

## 🔒 8. Neutrality Principle and Multi-User Architecture

> [!IMPORTANT]
> **Atena Studio is open-source, multi-user, and distributed software.**
> 
> - **Zero Hardcoded Assumptions:** The AI **never** contains pre-programmed real names, developer preferences, or specific user identities in its codebase or prompts.
> - **Dynamic Learning via `User`:** The identity of whoever operates the software is acquired exclusively at runtime via the `User` anchor node.
> - **100% Local Privacy:** User data is stored exclusively on the local machine where Atena Studio is installed (`~/.atena/`), completely isolated and under the user's sole control.
> - **Neutral Placeholders:** Examples in documentation, test suites, or prompts must always use neutral, fictional names (such as `Maria`, `Carlos`, `Ana`).

---

## 🚀 9. Conclusion

This architecture provides genuine long-term intelligence for Atena: it **remembers who you are** (Facts) and **learns how you like tasks performed** (Skills), continuously refining its methods over time in a simple, lightweight, and hardware-friendly manner.
