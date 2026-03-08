# cmdtyper v0.2 — 新版架构规划

> 从"打字练习工具"进化为"Linux 命令行交互式教学系统"

---

## 一、总体设计理念

### 定位转变

| | v0.1（当前） | v0.2（新版） |
|---|---|---|
| 定位 | 命令打字练习器 | Linux 命令行交互式教学系统 |
| 学习模式 | 显示注释 + 跟打（和对着打几乎相同） | 体系化教学：命令专题 / 符号专题 / 系统架构 / 模拟输出 |
| 对着打 | 显示命令 + 注释树 + 逐字符 | 模拟真实终端：prompt + 命令 + Enter 换行 |
| 内容深度 | 题库只有命令和注释 | 命令讲解 + 符号解释 + 目录结构 + 模拟输出 + 配置文件 |

### 核心原则

1. **学习模式做厚** — 成为真正的 Linux 入门教材，有体系、有深度、有反馈
2. **对着打做薄** — 极简终端模拟，沉浸式打字体验
3. **不执行真实命令** — 所有输出都是预置的模拟数据，安全第一
4. **模块化数据** — 教学内容与代码分离，用 TOML/Markdown 定义课程

---

## 二、模块总览

```
主菜单
├── ⌨️  对着打（Type）         ← 极简终端模拟
├── 📖 学习中心（Learn）       ← 二级菜单，体系化教学
│   ├── 命令专题
│   │   ├── 文件操作 (ls, cd, cp, mv, rm, ...)
│   │   ├── 文本处理 (grep, sed, awk, ...)
│   │   ├── 进程管理 (ps, kill, top, ...)
│   │   ├── 网络工具 (curl, ss, ip, ...)
│   │   ├── 权限与用户 (chmod, chown, sudo, ...)
│   │   ├── 压缩归档 (tar, gzip, zip, ...)
│   │   ├── Git 工作流 (git add, commit, push, ...)
│   │   └── ... (按 category 自动生成)
│   ├── 符号专题
│   │   ├── 管道与重定向 (|, >, >>, 2>&1, tee)
│   │   ├── 通配符 (*, ?, [], {})
│   │   ├── 引号与转义 (', ", \, $())
│   │   ├── 变量与展开 ($VAR, ${VAR}, $())
│   │   ├── 特殊字符 (;, &&, ||, &, !)
│   │   └── 正则表达式基础 (., *, +, ^, $, [])
│   ├── 系统架构
│   │   ├── Linux 目录结构 (/etc, /var, /usr, /home, ...)
│   │   ├── 文件系统与权限模型
│   │   ├── 进程与服务管理 (systemd)
│   │   ├── 网络配置基础
│   │   ├── 包管理 (apt/yum/pacman)
│   │   └── 配置文件实战 (bashrc, sshd_config, nginx.conf, ...)
│   └── 专题复习
│       └── (每个专题结束后自动出现的梳理 + 集中练习)
├── 📝 默写模式（Dictation）   ← 保持现有
├── 📊 统计面板（Stats）       ← 保持现有
└── ⚙️  设置（Settings）       ← 新增
```

---

## 三、对着打模式（Type） — 终端模拟器风格

### 3.1 设计目标

**像在真正的终端里打命令一样。**

- 屏幕顶部不再显示"对着打"标题栏
- 显示一个模拟的终端 prompt
- 灰色显示待输入的命令
- 用户逐字符输入，正确变白/绿
- 按 Enter 完成当前命令 → 下方出现下一条灰色命令
- 已完成的命令留在屏幕上（像真正的终端历史）

### 3.2 UI 布局

```
┌─────────────────────────────────────────────┐
│ user@cmdtyper:~$                            │  ← prompt（固定）
│                                             │
│ user@cmdtyper:~$ ls -la /var/log            │  ← 已完成（白色/绿色）
│ user@cmdtyper:~$ grep -r "error" /v█r/log   │  ← 当前正在输入（光标在 █）
│                                             │
│                                             │
│                                             │
│                                             │
│                                             │
├─────────────────────────────────────────────┤
│ 显示 /var/log 目录的详细列表    [H 隐藏提示] │  ← 命令含义提示（可关闭）
│                         WPM: 42  准确率: 96% │
└─────────────────────────────────────────────┘
```

### 3.3 交互流程

```
1. 进入对着打 → 屏幕显示第一个 prompt + 灰色命令
2. 用户逐字符输入
   - 正确：字符变白，光标右移
   - 错误：闪红，不前进
3. 输入完所有字符 → 按 Enter（或自动）
   - 当前行变为「已完成」样式
   - 光标移到下一行新 prompt
   - 下一条灰色命令出现
4. 终端历史向上滚动（超出屏幕的旧命令自动消失）
5. 底栏始终显示实时 WPM / 准确率
6. 含义提示默认显示，按 H 键切换显隐
```

### 3.4 多行命令支持

对于带管道的长命令，用 `\` 换行显示：

```
user@cmdtyper:~$ find /var/log -name "*.log" -mtime +30 \
>     -exec gzip {} \;
```

数据格式中新增 `display` 字段控制换行：

```toml
[[commands]]
id = "find-gzip-old-logs"
command = "find /var/log -name \"*.log\" -mtime +30 -exec gzip {} \\;"
display = """find /var/log -name "*.log" -mtime +30 \\
    -exec gzip {} \\;"""
```

### 3.5 数据变更

```toml
# 新增字段（可选）
[[commands]]
display = "..."          # 终端中的多行显示格式（可选，默认 = command）
summary_short = "..."    # 一句话含义（用于底栏提示，可选，默认 = summary）
```

---

## 四、学习中心（Learn） — 体系化教学

### 4.1 二级菜单结构

```
📖 学习中心
┌─────────────────────────────────────────────┐
│                                             │
│  ▸ 命令专题                                  │
│    按类别系统学习 Linux 命令                   │
│                                             │
│    符号专题                                   │
│    管道、重定向、通配符、引号、变量...           │
│                                             │
│    系统架构                                   │
│    目录结构、权限模型、服务管理、配置文件...      │
│                                             │
│    专题复习                                   │
│    回顾已学内容，集中强化练习                   │
│                                             │
│                                             │
│  ↑↓ 选择  Enter 进入  Esc 返回               │
└─────────────────────────────────────────────┘
```

选择"命令专题"后进入三级菜单：

```
📖 学习中心 › 命令专题
┌─────────────────────────────────────────────┐
│                                             │
│  ▸ 📁 文件操作          ★☆☆☆  进度 3/15    │
│    📋 文本处理          ★★★☆  进度 0/12    │
│    🔍 搜索查找          ★★☆☆  进度 0/8     │
│    ⚙️  进程管理          ★★☆☆  进度 0/10    │
│    🌐 网络工具          ★★★☆  进度 0/13    │
│    🔒 权限与用户        ★★☆☆  进度 0/9     │
│    📦 压缩归档          ★★☆☆  进度 0/7     │
│    💻 系统信息          ★★☆☆  进度 0/6     │
│    🔀 管道与重定向      ★★★☆  进度 0/11    │
│    📜 脚本片段          ★★★★  进度 0/8     │
│                                             │
│  ↑↓ 选择  Enter 进入  Esc 返回               │
└─────────────────────────────────────────────┘
```

### 4.2 命令专题 — 单个命令的学习流程

选择一个类别后，进入该类别的命令列表，选择一个命令进入学习：

```
阶段 1: 命令概览（读）
┌─────────────────────────────────────────────┐
│ 📖 命令学习: ls                              │
│                                             │
│  ls — 列出目录内容                           │
│                                             │
│  ls 是最常用的 Linux 命令之一。它用于显示      │
│  指定目录中的文件和子目录列表。               │
│                                             │
│  基本语法:                                   │
│    ls [选项] [路径]                           │
│                                             │
│  常用选项:                                   │
│    -l    详细列表格式（权限、大小、时间）      │
│    -a    显示隐藏文件（以 . 开头）             │
│    -h    人类可读的文件大小（KB/MB/GB）        │
│    -R    递归列出子目录                       │
│    -S    按文件大小排序                       │
│    -t    按修改时间排序                       │
│                                             │
│              Enter 继续 →                    │
└─────────────────────────────────────────────┘

阶段 2: 实例演示 + 模拟输出（看）
┌─────────────────────────────────────────────┐
│ 📖 命令学习: ls -la /var/log                 │
│                                             │
│  含义: 显示 /var/log 下所有文件的详细信息       │
│                                             │
│  各部分解释:                                  │
│  ├─ ls        列出目录内容的命令              │
│  ├─ -la       -l 详细格式 + -a 含隐藏文件     │
│  └─ /var/log  系统日志目录                    │
│                                             │
│  模拟输出:                                    │
│  ┌─────────────────────────────────────────┐ │
│  │ $ ls -la /var/log                       │ │
│  │ total 2048                              │ │
│  │ drwxr-xr-x  12 root root  4096 Mar  8  │ │
│  │ -rw-r-----   1 root adm  15234 Mar  8  │ │
│  │ -rw-r--r--   1 root root  8291 Mar  7  │ │
│  │ ...                                     │ │
│  └─────────────────────────────────────────┘ │
│                                             │
│              Enter 继续 →                    │
└─────────────────────────────────────────────┘

阶段 3: 跟打练习（打）
┌─────────────────────────────────────────────┐
│ 📖 跟打练习: ls -la /var/log                 │
│                                             │
│  $ ls -la /var/log                           │
│                                             │
│  ├─ ls        列出目录内容的命令              │
│  ├─ -la       -l 详细格式 + -a 含隐藏文件     │
│  └─ /var/log  系统日志目录                    │
│                                             │
│  跟着输入:                                    │
│  $ ls█-la /var/log                           │
│                                             │
│  模拟输出:                                    │
│  ┌─────────────────────────────────────────┐ │
│  │ total 2048                              │ │
│  │ drwxr-xr-x  12 root root  4096 ...     │ │
│  └─────────────────────────────────────────┘ │
│                                             │
│  WPM: 35  准确率: 98%  Tab 跳过  ^R 重练     │
└─────────────────────────────────────────────┘
```

**打完按 Enter 后，下方出现模拟输出 → 让用户直观看到"我刚才打的这条命令会产生什么效果"。**

### 4.3 符号专题 — 数据结构

新增符号专题数据文件：

```toml
# data/symbols/pipe_redirect.toml

[meta]
topic = "管道与重定向"
description = "理解数据流：如何让命令之间协作"
difficulty = "basic"
icon = "🔀"

[[symbols]]
id = "pipe"
char = "|"
name = "管道"
summary = "把前一个命令的输出，作为后一个命令的输入"
explanation = """
管道 | 是 Linux 最强大的概念之一。它让你把多个简单命令
组合成强大的数据处理流水线。

数据流向: 命令A 的 stdout → 命令B 的 stdin

就像工厂流水线：
  原材料 → [切割机] → [打磨机] → [喷漆机] → 成品
  原始数据 → [grep] → [sort] → [uniq] → 最终结果
"""

[[symbols.examples]]
command = "cat /etc/passwd | grep root"
explanation = "读取密码文件，然后从中筛选包含 root 的行"
simulated_output = """root:x:0:0:root:/root:/bin/bash"""

[[symbols.examples]]
command = "ls -la | wc -l"
explanation = "列出文件，然后统计行数（即文件数量）"
simulated_output = """42"""

[[symbols.examples]]
command = "cat access.log | grep 404 | sort | uniq -c | sort -rn | head -10"
explanation = "从访问日志中找出出现次数最多的 10 个 404 错误页面"
display = """cat access.log | grep 404 \\
    | sort | uniq -c \\
    | sort -rn | head -10"""
simulated_output = """    127 /old-page.html
     84 /deleted-image.png
     53 /api/v1/legacy
     ..."""

# 每个符号专题末尾的集中练习
[[exercises]]
prompt = "把 ls 的输出传给 wc -l 来统计文件数量"
answers = ["ls | wc -l", "ls -1 | wc -l"]

[[exercises]]
prompt = "从 /etc/passwd 中筛选出包含 'bash' 的行"
answers = ["cat /etc/passwd | grep bash", "grep bash /etc/passwd"]
```

### 4.4 系统架构专题 — 数据结构

```toml
# data/system/directory_structure.toml

[meta]
topic = "Linux 目录结构"
description = "理解 Linux 文件系统的骨架"
difficulty = "beginner"
icon = "🏗️"

# 用 ASCII art 展示目录树
overview = """
/                     根目录，一切的起点
├── bin/              基础命令 (ls, cp, grep...)
├── etc/              系统配置文件
│   ├── nginx/        Nginx 配置
│   ├── ssh/          SSH 配置
│   └── systemd/      服务管理配置
├── home/             用户主目录
│   └── alice/        用户 alice 的家
├── var/              可变数据
│   ├── log/          系统日志
│   └── www/          Web 文件
├── usr/              用户程序
│   ├── bin/          用户命令
│   └── lib/          共享库
├── tmp/              临时文件（重启清空）
├── dev/              设备文件
├── proc/             进程信息（虚拟文件系统）
└── sys/              内核信息（虚拟文件系统）
"""

[[directories]]
id = "etc"
path = "/etc"
name = "系统配置目录"
description = """
/etc 是 Linux 系统的"控制中心"。几乎所有系统级配置文件都在这里。
名字来源于 "et cetera"（拉丁语：其他），但现在一般理解为
"Editable Text Configuration"。

重要子目录:
- /etc/nginx/     — Web 服务器配置
- /etc/ssh/       — SSH 服务配置
- /etc/systemd/   — 系统服务管理
- /etc/apt/       — 包管理器配置
- /etc/cron.d/    — 定时任务
"""

[[directories.commands]]
command = "ls /etc/"
summary = "查看 /etc 下有哪些配置文件和目录"
simulated_output = """acpi        cloud     fstab      hosts        logrotate.d  nginx     resolv.conf  ssh       systemd
alternatives cron.d    group      hostname     lsb-release  os-release  rsyslog.d    ssl       timezone
apt          crontab   gshadow    hosts.allow  machine-id   pam.d      securetty    sudoers   ufw
bash.bashrc  default   host.conf  hosts.deny   mke2fs.conf  passwd     shadow       sudoers.d update-motd.d"""

[[directories.commands]]
command = "cat /etc/hostname"
summary = "查看当前系统的主机名"
simulated_output = """cmdtyper-lab"""

[[directories.commands]]
command = "cat /etc/os-release"
summary = "查看操作系统版本信息"
simulated_output = """PRETTY_NAME="Ubuntu 24.04.1 LTS"
NAME="Ubuntu"
VERSION_ID="24.04"
VERSION="24.04.1 LTS (Noble Numbat)"
ID=ubuntu
ID_LIKE=debian"""

# 配置文件实战
[[directories.config_files]]
id = "etc-ssh-sshd-config"
path = "/etc/ssh/sshd_config"
name = "SSH 服务端配置"
description = "控制 SSH 远程登录的安全策略和行为"
sample_content = """# SSH Server Configuration
Port 22
PermitRootLogin prohibit-password
PasswordAuthentication yes
PubkeyAuthentication yes
MaxAuthTries 6
X11Forwarding yes
"""

[[directories.config_files.lessons]]
title = "禁用密码登录（只允许密钥登录）"
before = "PasswordAuthentication yes"
after = "PasswordAuthentication no"
explanation = """
将 PasswordAuthentication 从 yes 改为 no 后，
SSH 将只接受密钥认证，拒绝所有密码登录尝试。
这是服务器安全加固的第一步。

修改后需要重启 SSH 服务:
  sudo systemctl restart sshd
"""
practice_command = "sudo sed -i 's/PasswordAuthentication yes/PasswordAuthentication no/' /etc/ssh/sshd_config"
```

### 4.5 模拟输出系统

#### 数据格式

在现有命令数据中新增 `simulated_output` 字段：

```toml
[[commands]]
id = "ls-la-varlog"
command = "ls -la /var/log"
summary = "显示 /var/log 目录的详细列表（包含隐藏文件）"

# 新增：模拟输出
simulated_output = """total 2048
drwxrwxr-x  12 root syslog  4096 Mar  8 10:00 .
drwxr-xr-x  14 root root    4096 Feb  1 00:00 ..
-rw-r-----   1 root adm    15234 Mar  8 09:55 auth.log
-rw-r--r--   1 root root    8291 Mar  7 23:00 boot.log
-rw-r-----   1 root adm   102400 Mar  8 10:00 syslog
-rw-r-----   1 root adm    45678 Mar  8 08:30 kern.log
-rw-rw-r--   1 root utmp   29184 Mar  8 09:00 wtmp"""

# 新增：输出中的关键标注（可选）
[[commands.output_annotations]]
pattern = "drwxrwxr-x"
note = "d=目录, rwx=所有者可读写执行, r-x=组可读执行, r-x=其他人可读执行"

[[commands.output_annotations]]
pattern = "root adm"
note = "所有者=root, 所属组=adm"

[[commands.output_annotations]]
pattern = "15234"
note = "文件大小（字节）"
```

#### 显示时机

```
学习模式 阶段 2（实例演示）:
  直接在命令下方展示模拟输出

学习模式 阶段 3（跟打练习）:
  打完按 Enter → 模拟输出从下方"打印"出来（带逐行动画）

对着打模式:
  不显示模拟输出（保持极简）

默写模式:
  答对后可选展示模拟输出
```

### 4.6 终端模拟 prompt 风格

```
# 普通用户
user@cmdtyper:~$

# 需要 sudo 的命令
user@cmdtyper:~$ sudo systemctl restart nginx
[sudo] password for user: ********    ← 模拟密码输入（显示 *）

# 进入特定目录的命令
user@cmdtyper:/var/log$ ls -la

# 多行命令（\ 续行）
user@cmdtyper:~$ find /var/log -name "*.log" \
>     -mtime +30 -delete

# 管道命令（显示为单行或带 \ 的多行）
user@cmdtyper:~$ cat access.log | grep 404 | sort | uniq -c | sort -rn
```

prompt 样式可配置：

```toml
# data/config/terminal.toml (或在 UserConfig 里)
[terminal]
prompt_style = "full"       # "full" = user@host:path$ | "simple" = $ | "minimal" = >
username = "user"
hostname = "cmdtyper"
show_path = true            # 是否显示路径
```

### 4.7 专题复习模块

每个专题（命令/符号/系统架构）结束后，自动生成一个复习环节：

```
专题复习: 文件操作
┌─────────────────────────────────────────────┐
│ 📋 知识梳理                                  │
│                                             │
│  本节学习了 15 个文件操作命令:                 │
│                                             │
│  基础:                                       │
│    ls    列出目录    cd    切换目录            │
│    pwd   当前路径    mkdir 创建目录            │
│    touch 创建文件    cat   查看文件            │
│                                             │
│  操作:                                       │
│    cp    复制        mv    移动/重命名         │
│    rm    删除        ln    创建链接            │
│                                             │
│  查看:                                       │
│    head  看开头      tail  看结尾              │
│    less  分页浏览    wc    统计行数            │
│    file  查看类型                             │
│                                             │
│         Enter 进入集中练习 →                  │
└─────────────────────────────────────────────┘
```

集中练习：混合本专题所有命令，随机出题，快速打字 + 默写交替。

---

## 五、新数据架构

### 5.1 目录结构

```
data/
├── commands/               # 现有命令题库（保留，增强）
│   ├── 01_beginner.toml
│   ├── ...
│   └── 19_practical_oneliners.toml
├── symbols/                # 新增：符号专题
│   ├── pipe_redirect.toml
│   ├── wildcards.toml
│   ├── quotes_escape.toml
│   ├── variables.toml
│   ├── special_chars.toml
│   └── regex_basics.toml
├── system/                 # 新增：系统架构专题
│   ├── directory_structure.toml
│   ├── filesystem_permissions.toml
│   ├── process_systemd.toml
│   ├── network_basics.toml
│   ├── package_management.toml
│   └── config_files.toml
├── lessons/                # 新增：命令讲解内容
│   ├── ls.toml
│   ├── grep.toml
│   ├── find.toml
│   ├── awk.toml
│   └── ... (每个重要命令一个文件)
├── reviews/                # 新增：专题复习数据（自动生成或手写）
│   └── (按专题 ID 命名)
└── config/
    └── terminal.toml       # 终端模拟配置
```

### 5.2 命令讲解文件格式

```toml
# data/lessons/grep.toml

[meta]
command = "grep"
full_name = "Global Regular Expression Print"
category = "text_process"
difficulty = "basic"
importance = "core"             # core | common | advanced | niche
description = "在文件或输入中搜索匹配模式的行"

[overview]
summary = "grep 是文本搜索的瑞士军刀，几乎每天都会用到。"
explanation = """
grep 用于在文件（或标准输入）中搜索包含指定模式的行，
并把匹配的行输出到屏幕。

它的名字来源于 ed 编辑器的命令 g/re/p：
  g = global（全局）
  re = regular expression（正则表达式）
  p = print（打印）
"""

# 语法说明
[syntax]
basic = "grep [选项] 模式 [文件...]"
parts = [
    { name = "模式", desc = "要搜索的字符串或正则表达式" },
    { name = "文件", desc = "要搜索的文件（省略则从 stdin 读取）" },
]

# 常用选项
[[options]]
flag = "-i"
name = "忽略大小写"
example = "grep -i 'error' log.txt"
note = "Error、ERROR、error 都能匹配"

[[options]]
flag = "-r"
name = "递归搜索目录"
example = "grep -r 'TODO' src/"
note = "搜索 src/ 下所有文件"

[[options]]
flag = "-n"
name = "显示行号"
example = "grep -n 'main' *.py"
note = "输出 '文件名:行号:匹配内容'"

[[options]]
flag = "-c"
name = "只输出匹配行数"
example = "grep -c '404' access.log"
note = "不输出内容，只输出匹配了多少行"

[[options]]
flag = "-v"
name = "反向匹配"
example = "grep -v 'comment' config.txt"
note = "输出不包含 'comment' 的行"

[[options]]
flag = "-l"
name = "只输出文件名"
example = "grep -rl 'deprecated' src/"
note = "列出包含匹配的文件名，不显示具体行"

# 实战示例（由简到难）
[[examples]]
level = 1
command = "grep 'error' /var/log/syslog"
summary = "在系统日志中搜索 error"
simulated_output = """Mar  8 09:15:23 server kernel: [error] disk I/O timeout on /dev/sda
Mar  8 09:20:45 server nginx: [error] upstream timed out"""

[[examples]]
level = 2
command = "grep -rn 'TODO' src/"
summary = "递归搜索源代码中的 TODO 标记，显示行号"
simulated_output = """src/main.rs:42:    // TODO: add error handling
src/app.rs:156:    // TODO: implement settings page
src/ui/stats.rs:89:    // TODO: add sparkline chart"""

[[examples]]
level = 3
command = "grep -c '404' access.log"
summary = "统计访问日志中 404 错误的数量"
simulated_output = """273"""

[[examples]]
level = 4
command = "cat /etc/passwd | grep -v 'nologin' | grep -v 'false'"
summary = "找出所有可以登录的用户"
display = """cat /etc/passwd \\
    | grep -v 'nologin' \\
    | grep -v 'false'"""
simulated_output = """root:x:0:0:root:/root:/bin/bash
alice:x:1000:1000:Alice,,,:/home/alice:/bin/bash
deploy:x:1001:1001::/home/deploy:/bin/bash"""

# 易混淆点
[[gotchas]]
title = "grep vs fgrep vs egrep"
content = """
- grep    — 基础正则（BRE）
- egrep   — 扩展正则（ERE），等价于 grep -E
- fgrep   — 纯字符串匹配（不解释正则），等价于 grep -F

实际使用中推荐用 grep -E 代替 egrep，grep -F 代替 fgrep。
"""
```

### 5.3 Rust 数据模型新增

```rust
// src/data/models.rs 新增

/// 学习中心二级菜单项
pub enum LearnSection {
    CommandTopics,      // 命令专题
    SymbolTopics,       // 符号专题
    SystemTopics,       // 系统架构
    Review,             // 专题复习
}

/// 符号专题
pub struct SymbolTopic {
    pub id: String,
    pub topic: String,
    pub description: String,
    pub difficulty: Difficulty,
    pub icon: String,
    pub symbols: Vec<SymbolEntry>,
    pub exercises: Vec<Exercise>,
}

pub struct SymbolEntry {
    pub id: String,
    pub char_repr: String,          // "|", ">", etc.
    pub name: String,
    pub summary: String,
    pub explanation: String,        // 详细讲解（多行）
    pub examples: Vec<SymbolExample>,
}

pub struct SymbolExample {
    pub command: String,
    pub display: Option<String>,
    pub explanation: String,
    pub simulated_output: Option<String>,
}

/// 系统架构专题
pub struct SystemTopic {
    pub id: String,
    pub topic: String,
    pub description: String,
    pub difficulty: Difficulty,
    pub icon: String,
    pub overview: String,           // ASCII art 或总览文本
    pub directories: Vec<DirectoryEntry>,
}

pub struct DirectoryEntry {
    pub id: String,
    pub path: String,
    pub name: String,
    pub description: String,
    pub commands: Vec<DirCommand>,
    pub config_files: Vec<ConfigFile>,
}

pub struct DirCommand {
    pub command: String,
    pub summary: String,
    pub simulated_output: String,
}

pub struct ConfigFile {
    pub id: String,
    pub path: String,
    pub name: String,
    pub description: String,
    pub sample_content: String,
    pub lessons: Vec<ConfigLesson>,
}

pub struct ConfigLesson {
    pub title: String,
    pub before: String,
    pub after: String,
    pub explanation: String,
    pub practice_command: Option<String>,
}

/// 命令详细讲解
pub struct CommandLesson {
    pub command_name: String,
    pub full_name: Option<String>,
    pub category: Category,
    pub difficulty: Difficulty,
    pub importance: String,
    pub overview: LessonOverview,
    pub syntax: SyntaxInfo,
    pub options: Vec<OptionInfo>,
    pub examples: Vec<LessonExample>,
    pub gotchas: Vec<Gotcha>,
}

pub struct LessonOverview {
    pub summary: String,
    pub explanation: String,
}

pub struct SyntaxInfo {
    pub basic: String,
    pub parts: Vec<SyntaxPart>,
}

pub struct LessonExample {
    pub level: u8,
    pub command: String,
    pub display: Option<String>,
    pub summary: String,
    pub simulated_output: Option<String>,
}

pub struct Gotcha {
    pub title: String,
    pub content: String,
}

/// 模拟输出注解
pub struct OutputAnnotation {
    pub pattern: String,
    pub note: String,
}

/// 复习模块
pub struct ReviewData {
    pub topic_id: String,
    pub topic_name: String,
    pub summary_groups: Vec<ReviewGroup>,    // 知识梳理分组
    pub practice_commands: Vec<String>,      // 集中练习用的命令 ID 列表
}

pub struct ReviewGroup {
    pub name: String,                        // "基础", "操作", "查看"
    pub items: Vec<ReviewItem>,
}

pub struct ReviewItem {
    pub command: String,
    pub brief: String,                       // 极短描述（4-6字）
}
```

---

## 六、App 状态机改造

### 6.1 新状态

```rust
pub enum AppState {
    Home,                                       // 主菜单
    // ── 对着打 ──
    Typing,                                     // 终端模拟风格打字
    // ── 学习中心 ──
    LearnMenu,                                  // 二级菜单
    LearnCommandTopics,                         // 命令专题列表
    LearnCommandLesson(LessonPhase),            // 单个命令学习（三阶段）
    LearnSymbolTopics,                          // 符号专题列表
    LearnSymbolLesson,                          // 单个符号学习
    LearnSystemTopics,                          // 系统架构专题列表
    LearnSystemLesson,                          // 单个系统主题学习
    LearnReview,                                // 专题复习
    // ── 其他 ──
    Dictation,                                  // 默写
    Stats,                                      // 统计
    Settings,                                   // 设置（新增）
    RoundResult,                                // 本轮结果
    Quitting,
}

pub enum LessonPhase {
    Overview,           // 阶段 1: 命令概览（读）
    Examples(usize),    // 阶段 2: 实例演示 + 模拟输出（看），usize = 当前示例索引
    Practice(usize),    // 阶段 3: 跟打练习（打），usize = 当前示例索引
}
```

### 6.2 导航流程图

```
Home
 ├─→ Typing ──→ (终端模拟打字，Esc 返回)
 ├─→ LearnMenu
 │    ├─→ LearnCommandTopics
 │    │    └─→ LearnCommandLesson
 │    │         ├─ Overview → Examples → Practice → (下一个命令或返回)
 │    │         └─ 专题结束 → LearnReview
 │    ├─→ LearnSymbolTopics
 │    │    └─→ LearnSymbolLesson → ... → LearnReview
 │    ├─→ LearnSystemTopics
 │    │    └─→ LearnSystemLesson → ... → LearnReview
 │    └─→ LearnReview（直接进入复习）
 ├─→ Dictation
 ├─→ Stats
 └─→ Settings
```

---

## 七、开发分期

### Phase A: 对着打改造（终端模拟风格）

**范围**: 改造 `src/ui/typing.rs` + `src/app.rs` 部分状态

1. 去掉顶栏标题
2. 实现 prompt 渲染 (`user@cmdtyper:~$`)
3. 实现终端历史滚动（已完成的命令留在屏幕上）
4. 按 Enter 完成当前命令 → 新行新命令
5. 底栏含义提示 + H 键切换
6. 底栏实时 WPM / 准确率
7. 多行命令 display 支持
8. 数据层：commands 新增 `display`、`summary_short` 可选字段

### Phase B: 学习中心骨架 + 命令专题

**范围**: 新建 `src/ui/learn_menu.rs`、`src/ui/learn_command.rs`，改造 `src/app.rs` 状态机

1. 学习中心二级菜单 UI
2. 命令专题三级菜单（按 category 自动生成列表）
3. 单命令学习三阶段 UI（概览 → 演示 → 跟打）
4. 模拟输出渲染组件（终端框 + 内容）
5. 数据层：新建 `data/lessons/*.toml`，实现 loader
6. 核心命令讲解内容编写（ls, cd, grep, find, chmod, ps, curl 等 ~15 个）

### Phase C: 符号专题

1. 符号专题菜单 + 学习 UI
2. 数据层：`data/symbols/*.toml`
3. 内容编写（管道、重定向、通配符、引号、变量、特殊字符）
4. 符号专题练习题

### Phase D: 系统架构专题

1. 系统架构菜单 + 学习 UI
2. 目录结构 ASCII art 展示
3. 目录探索交互（选目录 → 看说明 → 看常用命令 → 模拟输出）
4. 配置文件讲解 + before/after 对比展示
5. 数据层：`data/system/*.toml`
6. 内容编写

### Phase E: 专题复习 + 集中练习

1. 知识梳理页面渲染
2. 集中练习模式（混合打字 + 默写）
3. 按专题自动生成复习数据
4. 与统计系统集成（追踪每个专题的掌握度）

### Phase F: 设置页面 + 细节打磨

1. 设置页面（终端 prompt 风格、提示开关、目标 WPM）
2. 终端 prompt 可配置
3. UI 打磨（过渡动画、颜色统一）
4. 全部模式集成测试
5. README 更新

---

## 八、内容编写计划

### 命令讲解（Phase B，优先级排序）

| 优先级 | 命令 | 分类 |
|--------|------|------|
| P0 核心 | ls, cd, pwd, cat, echo, cp, mv, rm, mkdir | file_ops |
| P0 核心 | grep, find, chmod, chown, sudo | search/permission |
| P1 常用 | head, tail, less, wc, sort, uniq | text_process |
| P1 常用 | ps, kill, top, systemctl | process |
| P1 常用 | tar, gzip, zip | archive |
| P2 进阶 | awk, sed, xargs, cut, tr | text_process |
| P2 进阶 | curl, wget, ss, ip, ping | network |
| P2 进阶 | git (add, commit, push, pull, branch) | scripting |
| P3 实战 | docker, rsync, ssh, scp, crontab | practical |

### 符号讲解（Phase C）

| 文件 | 内容 |
|------|------|
| pipe_redirect.toml | `\|` `>` `>>` `2>` `2>&1` `<` `<<` `tee` |
| wildcards.toml | `*` `?` `[]` `{}` |
| quotes_escape.toml | `'...'` `"..."` `\` `` `...` `` `$(...)` |
| variables.toml | `$VAR` `${VAR}` `$()` `$?` `$!` `$#` `$@` `$$` |
| special_chars.toml | `;` `&&` `\|\|` `&` `!` `#` `~` `.` |
| regex_basics.toml | `.` `*` `+` `?` `^` `$` `[]` `()` `\d` `\w` |

### 系统架构（Phase D）

| 文件 | 内容 |
|------|------|
| directory_structure.toml | FHS 标准目录 + 各目录用途 |
| filesystem_permissions.toml | rwx 模型、umask、SUID/SGID |
| process_systemd.toml | 进程生命周期、systemd unit 文件 |
| network_basics.toml | ifconfig/ip、DNS、端口、iptables/ufw |
| package_management.toml | apt/yum/pacman 对比 |
| config_files.toml | bashrc, sshd_config, nginx.conf, fstab |

---

## 九、工作量预估

| Phase | 代码量（估） | 内容量（估） | 复杂度 |
|-------|-------------|-------------|--------|
| A: 对着打改造 | ~300 行改动 | 少量字段新增 | ★★☆ |
| B: 命令专题 | ~1500 行新增 | ~15 个命令讲解文件 | ★★★ |
| C: 符号专题 | ~800 行新增 | 6 个符号讲解文件 | ★★☆ |
| D: 系统架构 | ~1200 行新增 | 6 个系统讲解文件 | ★★★ |
| E: 专题复习 | ~600 行新增 | 自动生成为主 | ★★☆ |
| F: 设置 + 打磨 | ~400 行新增 | 无 | ★☆☆ |

**总计**: ~4800 行 Rust 新增/改动 + ~30 个 TOML 内容文件

---

## 十、安全声明（保持不变）

**cmdtyper 永远不执行任何真实命令。**

所有"命令输出"都是预置在 TOML 文件中的模拟数据。没有 shell 调用、没有 `Command::new()`、没有 `system()`。配置文件的"修改练习"也是纯文本对比展示，不会真正修改任何系统文件。
