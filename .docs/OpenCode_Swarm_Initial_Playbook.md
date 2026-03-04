# OpenCode 蜂群开发行动指南

> **版本**: 1.0  
> **适用**: 软件工程 3.5（交互即智能）  
> **核心平台**: OpenCode + MCP/A2A 协议栈  
> **组织形态**: 分布式蜂群智能（Swarm Intelligence）

---

## 目录

1. [蜂群架构总览](#一蜂群架构总览)
2. [认知层架构](#二认知层架构)
3. [蜂群兵种手册](#三蜂群兵种手册)
4. [MCP 工具链](#四mcp-工具链)
5. [技能矩阵](#五技能矩阵)
6. [作战流程](#六作战流程)
7. [项目启动协议](#七项目启动协议)
8. [蜂群十二条军规](#八蜂群十二条军规)

---

## 一、蜂群架构总览

### 1.1 军团构成

OpenCode 蜂群由 **1 个指挥中心** + **N 个战术 Agent** + **6 个 MCP 服务器** + **27 个技能**构成：

```yaml
# 蜂群单元配置
swarm_cell:
  version: "1.0"
  
  # 核心配置（ ~/.config/opencode/opencode.json ）
  commander:
    primary: "build"           # 构建指挥官（协调实施）
    planner: "plan"            # 规划指挥官（任务拆解）
    strategist: "sisyphus"     # oh-my-opencode 战略指挥官
    
  # 基础战术 Agent（7 个自定义 Agent）
  tactical_agents:
    - coder:      "工程兵"      # 代码实现
    - researcher: "侦察兵"      # 外部研究
    - reviewer:   "审查兵"      # 代码审查
    - scribe:     "文书兵"      # 文档撰写
    - explore:    "探索兵"      # 代码探索
    - plan:       "规划兵"      # 任务规划
    - build:      "指挥官"      # 构建协调
    
  # oh-my-opencode 扩展 Agent（10 个）
  omo_agents:
    - sisyphus:          "战略指挥官"
    - hephaestus:        "深度工程兵"
    - prometheus:        "规划专家"
    - oracle:            "架构顾问"
    - librarian:         "知识管理员"
    - explore:           "代码探索者"
    - multimodal-looker: "多模态分析"
    - metis:             "策略顾问"
    - momus:             "审查专家"
    - atlas:             "基础设施"
    
  # MCP 工具链（6 个服务器）
  mcp_servers:
    - MiniMax:      "AI 模型服务"
    - filesystem:   "文件系统访问"
    - context7:     "文档查询"
    - exa:          "网络搜索"
    - gh_grep:      "GitHub 代码搜索"
    - playwright:   "浏览器自动化"
    
  # 技能库（27 个技能）
  skills:
    custom: 5           # 自定义技能
    omo: 5              # oh-my-opencode 技能
    claude: 17          # Claude 官方技能
```

### 1.2 模型配置

当前使用模型：
- **主力模型**: `minimax-cn-coding-plan/MiniMax-M2.5-highspeed`（oh-my-opencode 专用）
- **API 后端**: DeepSeek（通过 OPENAI_BASE_URL 配置）
- **本地模型**: Ollama（M3 Max + 64GB 优化配置）

---

## 二、认知层架构

### 2.1 三层认知模型

```
Layer 3: 战略认知（Strategic）
├─ Commander: build / sisyphus
├─ 长期规划: Roadmap / Architecture Decision
└─ 资源调度: Agent 分配 / 任务分解

Layer 2: 战术认知（Tactical）
├─ 各兵种 Agent 并行执行
├─ 实时协同: MCP 协议 / A2A 通信
└─ 冲突消解: 权限边界 / 共识机制

Layer 1: 执行认知（Execution）
├─ MCP 工具链: Git / Docker / Terminal / Filesystem
├─ 本地 LLM 推理: Ollama
└─ 状态持久化: SQLite / ChromaDB
```

### 2.2 权限边界

**全局权限（默认拒绝）**:
```json
{
  "task": "deny",
  "context7_*": "deny",
  "exa_*": "deny",
  "gh_grep_*": "deny",
  "webfetch": "deny",
  "worktree_*": "deny"
}
```

**Agent 级别按需授权**:
- `researcher`: context7/exa/gh_grep/webfetch（外部研究）
- `coder`: read/write/edit/bash（代码实现）
- `reviewer`: git 命令只读（代码审查）
- `plan`/`build`: task/worktree（任务管理）

---

## 三、蜂群兵种手册

### 3.1 基础战术 Agent（7 个）

#### 🔧 Coder（工程兵）
- **模式**: subagent（叶子节点）
- **核心使命**: 代码实现与修复
- **权限**: read/write/edit/bash（代码相关）
- **必须技能**: 
  - `code-philosophy`（后端逻辑）
  - `frontend-philosophy`（前端 UI）
- **工作流程**:
  1. Read - 理解任务，读取相关文件
  2. Load Philosophy - 加载对应技能
  3. Plan - 制定实现策略
  4. Implement - 编写/修改代码
  5. Verify - 运行 lint/type-check/tests
  6. Checklist - 核对哲学清单
  7. Return - 提供变更摘要
- **禁止事项**:
  - 不得提交代码（git commit）
  - 不得编写测试（除非明确指令）
  - 不得进行外部研究
  - 不得撰写文档
  - 不得生成子 Agent

#### 🔍 Researcher（侦察兵）
- **模式**: subagent
- **核心使命**: 外部知识收集
- **权限**: context7/exa/gh_grep/webfetch + 只读 bash
- **输出要求**:
  - 过度详细（excessively detailed）
  - 包含完整可复制的代码片段
  - 每个发现必须有引用（citation）
  - 实现就绪（implementation-ready）
- **禁止事项**:
  - 不得写入文件
  - 不得修改文件系统
  - 不得返回无代码的摘要

#### 📝 Scribe（文书兵）
- **模式**: subagent
- **核心使命**: 文档撰写与维护
- **权限**: read/write/edit/glob
- **禁止**: bash（所有命令）

#### 🔎 Reviewer（审查兵）
- **模式**: subagent
- **核心使命**: 代码审查
- **温度**: 0.1（低创意，高一致）
- **权限**: git 命令只读 + read
- **禁止**: write/edit

#### 🗺️ Explore（探索兵）
- **模式**: subagent
- **核心使命**: 代码库探索与分析
- **权限**: 只读 bash 命令（ls/cat/grep/git 等）
- **禁止**: write/edit

#### 📋 Plan（规划兵）
- **模式**: subagent
- **核心使命**: 任务规划与分解
- **权限**: task/worktree
- **禁止**: edit/write/bash

#### 🎯 Build（指挥官）
- **模式**: primary
- **核心使命**: 构建协调与实施管理
- **关键约束**: 
  - 不得直接编辑文件
  - 不得直接运行命令
  - 所有实现必须委托给 `coder`
- **职责**:
  - 委托实现给 `coder`
  - 委托文档给 `scribe`
  - 委托分析给 `explore`
  - 委托研究给 `researcher`
  - 解释结果并决定下一步

### 3.2 oh-my-opencode 扩展 Agent（10 个）

#### 🎖️ Sisyphus（战略指挥官）
- **模型**: MiniMax-M2.5-highspeed
- **模式**: primary
- **技能**: file-operations, code-analysis, document-generation, web-research, data-processing
- **核心能力**:
  -  obsessively planning with todos
  -  assess search complexity before exploration
  -  delegate strategically via category+skills combinations
  -  use explore for internal code, librarian for external docs

#### 🔨 Hephaestus（深度工程兵）
- **模型**: MiniMax-M2.5-highspeed
- **模式**: primary
- **技能**: file-operations, code-analysis, document-generation, web-research, data-processing
- **核心能力**:
  - 自主深度工作者
  - 端到端完成任务
  - 探索后行动
  - 使用 explore/librarian agents 获取上下文

#### 🔮 Prometheus（规划专家）
- **模型**: MiniMax-M2.5-highspeed
- **模式**: all（所有模式）
- **核心能力**:
  - 战略规划顾问
  - 需求澄清
  - 工作规划（保存到 `.sisyphus/plans/*.md`）
  - **关键约束**: 只规划，不实施

#### 📚 Oracle（架构顾问）
- **模型**: MiniMax-M2.5-highspeed
- **核心能力**: 架构决策咨询

#### 📖 Librarian（知识管理员）
- **模型**: MiniMax-M2.5-highspeed
- **核心能力**: 文档查询与知识检索

#### 🌐 Explore（代码探索者）
- **模型**: MiniMax-M2.5-highspeed
- **核心能力**: 代码库内部探索

#### 🖼️ Multimodal-Looker（多模态分析）
- **模型**: MiniMax-M2.5-highspeed
- **核心能力**: 多模态内容分析

#### 🧠 Metis（策略顾问）
- **模型**: MiniMax-M2.5-highspeed
- **核心能力**: 策略建议

#### 👁️ Momus（审查专家）
- **模型**: MiniMax-M2.5-highspeed
- **核心能力**: 计划审查

#### 🗺️ Atlas（基础设施）
- **模型**: MiniMax-M2.5-highspeed
- **核心能力**: 基础设施管理

---

## 四、MCP 工具链

### 4.1 已配置的 MCP 服务器（6 个）

```json
{
  "mcp": {
    "MiniMax": {
      "type": "local",
      "command": ["uvx", "minimax-coding-plan-mcp", "-y"],
      "enabled": true,
      "environment": {
        "MINIMAX_API_HOST": "https://api.minimaxi.com"
      }
    },
    "filesystem": {
      "type": "local",
      "command": ["node", "/path/to/server-filesystem/dist/index.js", "/Users/gq"],
      "enabled": true
    },
    "context7": {
      "type": "local",
      "command": ["npx", "-y", "@upstash/context7-mcp", "--api-key", "..."],
      "enabled": true
    },
    "exa": {
      "type": "remote",
      "url": "https://mcp.exa.ai/mcp",
      "enabled": true
    },
    "gh_grep": {
      "type": "remote",
      "url": "https://mcp.grep.app",
      "enabled": true
    },
    "playwright": {
      "type": "local",
      "command": "npx",
      "args": ["-y", "@playwright/mcp@latest"],
      "enabled": true
    }
  }
}
```

### 4.2 工具链使用指南

| MCP | 用途 | 使用 Agent | 权限 |
|-----|------|-----------|------|
| MiniMax | AI 模型调用 | 所有 | allow |
| filesystem | 文件系统操作 | coder, scribe | allow |
| context7 | 文档查询 | researcher | allow |
| exa | 网络搜索 | researcher | allow |
| gh_grep | GitHub 代码搜索 | researcher | allow |
| playwright | 浏览器自动化 | 按需 | deny |

---

## 五、技能矩阵

### 5.1 技能分类（27 个）

#### 🎯 核心开发技能（7 个）
| 技能 | 来源 | 用途 | 绑定 Agent |
|------|------|------|-----------|
| code-philosophy | 自定义 | 代码哲学（5 Laws） | coder |
| code-review | 自定义 | 代码审查规范 | reviewer |
| frontend-philosophy | 自定义 | 前端设计哲学 | coder |
| code-analysis | OMO | 代码分析 | sisyphus, hephaestus |
| mcp-builder | Claude | MCP 开发 | 按需 |
| frontend-design | Claude | 前端设计 | 按需 |
| skill-creator | Claude | 技能创建 | 按需 |

#### 📄 文档处理技能（4 个）
| 技能 | 来源 | 用途 |
|------|------|------|
| docx | Claude | Word 文档 |
| pdf | Claude | PDF 处理 |
| xlsx | Claude | Excel 表格 |
| pptx | Claude | PPT 演示 |
| document-generation | OMO | 文档生成 |

#### 🎨 设计创意技能（5 个）
| 技能 | 来源 | 用途 |
|------|------|------|
| algorithmic-art | Claude | 算法艺术 |
| canvas-design | Claude | 视觉设计 |
| theme-factory | Claude | 主题样式 |
| brand-guidelines | Claude | 品牌指南 |
| frontend-design | Claude | 前端设计 |

#### 🌐 Web 开发技能（2 个）
| 技能 | 来源 | 用途 |
|------|------|------|
| web-artifacts-builder | Claude | React + Tailwind |
| webapp-testing | Claude | Web 测试 |

#### 🔧 基础设施技能（4 个）
| 技能 | 来源 | 用途 | 绑定 Agent |
|------|------|------|-----------|
| file-operations | OMO | 文件操作 | sisyphus, hephaestus |
| data-processing | OMO | 数据处理 | sisyphus, hephaestus |
| web-research | OMO | 网络研究 | sisyphus, hephaestus |
| plan-protocol | 自定义 | 规划协议 | plan |
| plan-review | 自定义 | 计划审查 | plan |

### 5.2 技能加载策略

**当前配置**:
```json
{
  "skills": {
    "enabled": true,
    "paths": ["/Users/gq/.config/opencode/skills"],
    "autoLoad": true
  }
}
```

**注意**: `autoLoad: true` 会加载所有 27 个技能，可能增加启动时间 1-3 秒。

---

## 六、作战流程

### 6.1 蜂群启动协议

```bash
# 1. 项目初始化
open /path/to/project

# 2. 读取本指南（自动通过 instructions 加载）

# 3. 启动蜂群指挥中心
# - build agent 自动激活
# - oh-my-opencode agents 并行加载

# 4. 建立共享认知空间
# - MCP 服务器初始化
# - SQLite 数据库连接
```

### 6.2 特征开发作战

#### Phase 1: 情报收集（Recon Phase）
- **Agent**: explore / researcher
- **任务**:
  1. 扫描现有代码库
  2. 识别依赖关系
  3. 标记高风险修改区域
- **输出**: 战场态势图（Heatmap）

#### Phase 2: 战术规划（Planning Phase）
- **Agent**: plan / prometheus / build
- **任务**:
  1. 分解任务为可并行子任务
  2. 生成分布式任务图（DAG）
  3. 分配 Agent 资源
- **输出**: 任务规划文档（`.sisyphus/plans/*.md`）

#### Phase 3: 协同突击（Execution Phase）

```yaml
assault_plan:
  wave_1:  # 工程兵建立基础结构
    agents: ["coder", "hephaestus"]
    tasks: ["schema_design", "api_skeleton", "feature_impl"]
    sync_point: "api_contract"
    
  wave_2:  # 狙击手优化关键路径
    agents: ["sniper"]
    dependencies: ["wave_1"]
    tasks: ["algorithm_optimization", "performance_tuning"]
    
  wave_3:  # 审查兵验收
    agents: ["reviewer", "momus"]
    tasks: ["code_review", "security_scan"]
    
  wave_4:  # 文书兵文档
    agents: ["scribe"]
    tasks: ["documentation", "changelog"]
```

#### Phase 4: 认知验收（Review Phase）
- **Agent**: reviewer + build
- **任务**:
  1. 自动化审查（代码规范、安全扫描）
  2. 架构一致性检查
  3. 人机协同验收

### 6.3 紧急响应作战

**蜂群自愈机制**:

1. **警报触发**: 监控系统 → Scout
2. **战情评估**: Scout 30 秒内定位故障
3. **紧急召集**: Commander 唤醒相关 Agent
4. **并行修复**:
   - Medic: 准备回滚方案（Plan B）
   - Engineer: 实施热修复（Plan A）
   - Sniper: 分析根因，防止复发
5. **认知沉淀**: 更新战例库

---

## 七、项目启动协议

### 7.1 自动加载的行动指南

**配置**: `~/.config/opencode/opencode.json`

```json
{
  "instructions": [
    "./tools/philosophy.md",
    "/Users/gq/projects/OpenCode_Swarm_Root_Guide.md"
  ]
}
```

**建议**: 在每个项目根目录创建 `.opencode/swarm-guide.md`:

```markdown
# 项目蜂群指南

## 项目元数据
- 名称: [项目名称]
- 类型: [Web/App/CLI/Library]
- 技术栈: [React/Node/Python/Go...]
- 规模: [小型/中型/大型]

## 兵种配置
- 主力工程兵: coder
- 主力侦察兵: researcher
- 审查标准: [严格/标准/宽松]

## 技能加载
- 必需: code-philosophy, frontend-philosophy
- 可选: web-artifacts-builder, mcp-builder

## 特殊规则
- [项目特定的约束]
- [命名规范]
- [架构决策]
```

### 7.2 启动检查清单

```markdown
## 蜂群启动检查清单

### 环境检查
- [ ] OpenCode 版本 >= 1.2.15
- [ ] MCP 服务器全部在线
- [ ] 技能加载完成（27 个）

### 项目检查
- [ ] Git 仓库已初始化
- [ ] 依赖安装完成
- [ ] 配置文件存在（.opencode/）

### Agent 检查
- [ ] Commander (build) 就绪
- [ ] 工程兵 (coder) 就绪
- [ ] 侦察兵 (researcher) 就绪

### 工具链检查
- [ ] Filesystem MCP 可访问项目目录
- [ ] Git MCP 可执行版本控制
- [ ] 网络 MCP 可用（如需外部资源）
```

---

## 八、蜂群十二条军规

1. **单点故障即失败**: 关键认知必须实时同步至共享记忆
2. **思维透明原则**: Agent 必须暴露思考过程（Chain-of-Thought）
3. **离线生存能力**: 蜂群单元必须能在断网环境下独立作战
4. **共识先于执行**: 重大架构变更需经共识，禁止单边行动
5. **人机边界清晰**: 人类负责战略目标（Why），Agent 负责战术执行（How）
6. **记忆继承义务**: Agent 退役前必须完整导出认知状态
7. **最小权限原则**: Scout 仅授予读权限，Engineer 按任务临时授予写权限
8. **回溯就绪**: 所有修改必须保留 Git 历史，支持任意时刻回溯
9. **异步优先**: 默认使用异步通信，阻塞同步仅用于关键共识
10. **涌现监控**: 持续监测蜂群整体智能水平（解决问题的速率）
11. **伦理审查**: 监控代码生成的法律/道德合规性
12. **和平时期演练**: 定期进行"混沌工程"，测试自愈能力

---

## 附录

### A. 术语表

- **MCP**: Model Context Protocol，模型上下文协议
- **A2A**: Agent-to-Agent，智能体间通信协议
- **CRDT**: Conflict-free Replicated Data Type，无冲突复制数据类型
- **RAG**: Retrieval-Augmented Generation，检索增强生成
- **OMO**: oh-my-opencode，扩展插件

### B. 配置文件清单

```
~/.config/opencode/
├── opencode.json              # 主配置（本指南基于此）
├── ocx.jsonc                 # 注册表配置
├── oh-my-opencode.json       # OMO 配置（10 Agents）
├── dcp.jsonc                 # DCP 插件配置
└── opencode-mem.jsonc       # 内存配置

~/.opencode/
├── agents/                   # 基础 Agent 定义
│   ├── coder.md
│   ├── researcher.md
│   ├── reviewer.md
│   └── scribe.md
├── skills/                   # 自定义技能
│   ├── code-philosophy/
│   ├── frontend-philosophy/
│   └── ...
└── plugin/                   # 本地插件
    ├── notify.ts
    ├── worktree.ts
    ├── background-agents.ts
    └── workspace-plugin.ts
```

### C. 快速参考卡片

```yaml
# 常用 Agent 调用
指挥官:    @build <task>
工程兵:    @coder <implementation>
侦察兵:    @researcher <research>
审查兵:    @reviewer <review>
规划兵:    @plan <planning>
探索兵:    @explore <exploration>

# 常用 MCP 工具
文件操作:  filesystem.read/write/edit
Git 操作:   bash.git
网络搜索:   exa.search
文档查询:   context7.query
代码搜索:   gh_grep.search

# 常用技能
代码哲学:   skill:code-philosophy
前端哲学:   skill:frontend-philosophy
代码分析:   skill:code-analysis
文件操作:   skill:file-operations
```

---

**文档结束**

> 本指南基于 OpenCode 1.2.15 配置，整合 17 个基础 Agent + 10 个 OMO Agent + 6 个 MCP + 27 个技能。
> 建议每个项目根目录放置 `.opencode/swarm-guide.md` 扩展项目特定规则。
