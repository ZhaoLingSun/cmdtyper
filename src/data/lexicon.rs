//! Token 通用解析库（Lexicon）
//!
//! 提供高频 token 的标准描述，作为 lesson token_details 缺失时的 fallback。
//! 支持按 token 文本查询，也支持在特定命令上下文中查询更精确的描述。

use std::collections::HashMap;
use std::sync::LazyLock;

/// 一条 lexicon 条目
#[derive(Debug, Clone)]
pub struct LexiconEntry {
    /// 默认解释
    pub desc: &'static str,
    /// 命令特定解释 (command_name -> desc)
    pub context: &'static [(&'static str, &'static str)],
}

static LEXICON: LazyLock<HashMap<&'static str, LexiconEntry>> = LazyLock::new(build_lexicon);

/// 查找 token 的标准解释（优先使用命令上下文）
pub fn lookup(token: &str, command_context: Option<&str>) -> Option<&'static str> {
    let entry = LEXICON.get(token)?;
    if let Some(cmd) = command_context {
        for &(ctx_cmd, ctx_desc) in entry.context {
            if cmd.starts_with(ctx_cmd) {
                return Some(ctx_desc);
            }
        }
    }
    Some(entry.desc)
}

/// 查找 token 的标准解释（无命令上下文）
pub fn lookup_simple(token: &str) -> Option<&'static str> {
    lookup(token, None)
}

macro_rules! lex {
    ($desc:expr) => {
        LexiconEntry { desc: $desc, context: &[] }
    };
    ($desc:expr, [ $( ($cmd:expr, $cdesc:expr) ),* $(,)? ]) => {
        LexiconEntry { desc: $desc, context: &[ $(($cmd, $cdesc)),* ] }
    };
}

fn build_lexicon() -> HashMap<&'static str, LexiconEntry> {
    let mut m = HashMap::new();

    // ── Shell 操作符 ──
    m.insert("|", lex!("管道符：将前一个命令的输出作为后一个命令的输入"));
    m.insert("||", lex!("逻辑或：前一个命令失败时才执行后一个命令"));
    m.insert("&&", lex!("逻辑与：前一个命令成功时才执行后一个命令"));
    m.insert(
        ";",
        lex!("命令分隔符：顺序执行多条命令，不管前一条是否成功"),
    );
    m.insert(">", lex!("输出重定向：将标准输出写入文件（覆盖已有内容）"));
    m.insert(">>", lex!("追加重定向：将标准输出追加到文件末尾"));
    m.insert("<", lex!("输入重定向：从文件读取内容作为标准输入"));
    m.insert("<<", lex!("Here Document：将后续文本块作为标准输入"));
    m.insert("2>", lex!("标准错误重定向：将错误输出写入文件"));
    m.insert("2>&1", lex!("将标准错误合并到标准输出"));
    m.insert("&", lex!("后台执行：让命令在后台运行"));
    m.insert(
        "{}",
        lex!(
            "占位符：被替换为当前处理的文件名",
            [
                ("find", "find 的占位符：代表当前匹配到的文件路径"),
                ("xargs", "xargs 的占位符：代表从标准输入读取的每一项"),
            ]
        ),
    );
    m.insert(
        "+",
        lex!(
            "批量执行终止符：尽量将多个参数合并为一次命令调用，减少进程启动开销",
            [
                (
                    "find",
                    "find -exec 的批量终止符：将匹配文件批量传给命令，而非逐个执行"
                ),
                ("chmod", "添加权限：在当前权限基础上增加指定权限"),
            ]
        ),
    );

    // ── 路径与位置 ──
    m.insert(".", lex!("当前目录"));
    m.insert("..", lex!("上级目录"));
    m.insert("~", lex!("用户主目录（通常是 /home/用户名）"));
    m.insert("/etc", lex!("系统配置文件目录"));
    m.insert("/var/log", lex!("系统日志目录"));
    m.insert("/var/www", lex!("Web 服务默认文档根目录"));
    m.insert(
        "/etc/passwd",
        lex!("用户账户信息文件（用户名、UID、主目录、Shell 等）"),
    );
    m.insert(
        "/dev/null",
        lex!("空设备：写入的数据被丢弃，读取立即返回 EOF"),
    );

    // ── 高频命令词 ──
    m.insert("find", lex!("在目录树中递归查找文件和目录"));
    m.insert("grep", lex!("在文件或输入中搜索匹配指定模式的行"));
    m.insert("sed", lex!("流编辑器：按规则对文本逐行进行查找替换或编辑"));
    m.insert("awk", lex!("文本处理语言：按字段分割和处理结构化文本"));
    m.insert("cut", lex!("按分隔符或字符位置提取文本的指定列"));
    m.insert("sort", lex!("对文本行进行排序"));
    m.insert("uniq", lex!("去除或统计相邻的重复行（通常配合 sort 使用）"));
    m.insert("xargs", lex!("将标准输入转换为命令参数并执行"));
    m.insert(
        "tar",
        lex!("归档工具：将多个文件打包为一个归档文件，或从归档中提取"),
    );
    m.insert("chmod", lex!("修改文件或目录的访问权限"));
    m.insert("chown", lex!("修改文件或目录的所有者和所属组"));
    m.insert(
        "systemctl",
        lex!("systemd 服务管理器：启动、停止、重启和查看服务状态"),
    );
    m.insert("journalctl", lex!("查看 systemd 日志（journal）"));
    m.insert("docker", lex!("容器管理工具：构建、运行和管理 Docker 容器"));
    m.insert("git", lex!("分布式版本控制系统"));
    m.insert("curl", lex!("命令行 HTTP 客户端：发送请求并获取响应"));
    m.insert("ps", lex!("显示当前系统中的进程信息"));
    m.insert("du", lex!("统计文件或目录的磁盘占用空间"));
    m.insert("df", lex!("显示文件系统的磁盘空间使用情况"));
    m.insert("ls", lex!("列出目录中的文件和子目录"));
    m.insert("cat", lex!("连接并显示文件内容"));
    m.insert("head", lex!("显示文件的前若干行"));
    m.insert("tail", lex!("显示文件的末尾若干行"));
    m.insert("echo", lex!("将文本输出到标准输出"));
    m.insert("cd", lex!("切换当前工作目录"));
    m.insert("pwd", lex!("打印当前工作目录的完整路径"));
    m.insert("mkdir", lex!("创建新目录"));
    m.insert("cp", lex!("复制文件或目录"));
    m.insert("mv", lex!("移动或重命名文件和目录"));
    m.insert("rm", lex!("删除文件或目录"));
    m.insert("touch", lex!("创建空文件，或更新已有文件的时间戳"));
    m.insert("ln", lex!("创建链接（硬链接或符号链接）"));
    m.insert("wc", lex!("统计文件的行数、单词数和字节数"));
    m.insert("tr", lex!("转换或删除字符"));
    m.insert("sudo", lex!("以超级用户（root）权限执行后续命令"));
    m.insert("kill", lex!("向进程发送信号（默认 SIGTERM 终止）"));
    m.insert("free", lex!("显示系统内存使用情况"));
    m.insert("rsync", lex!("高效的远程/本地文件同步工具"));
    m.insert("ssh", lex!("通过加密连接远程登录另一台主机"));
    m.insert("apt", lex!("Debian/Ubuntu 包管理器"));
    m.insert("rmdir", lex!("删除空目录"));
    m.insert(
        "jq",
        lex!("命令行 JSON 处理工具：解析、过滤和格式化 JSON 数据"),
    );

    // ── git 子命令 ──
    m.insert(
        "commit",
        lex!(
            "提交：将暂存区的更改记录为一个新版本",
            [("git", "将暂存区的更改保存为 Git 仓库中的一个新提交"),]
        ),
    );
    m.insert("push", lex!("推送：将本地提交同步到远程仓库"));
    m.insert("pull", lex!("拉取：从远程仓库获取更新并合并"));
    m.insert(
        "log",
        lex!(
            "查看提交历史记录",
            [("git", "显示 Git 提交历史"), ("journalctl", "显示日志"),]
        ),
    );
    m.insert(
        "status",
        lex!(
            "查看当前状态",
            [
                ("git", "显示工作区和暂存区的文件变更状态"),
                ("systemctl", "显示 systemd 服务的运行状态"),
            ]
        ),
    );
    m.insert(
        "diff",
        lex!("查看差异", [("git", "显示工作区与暂存区之间的文件差异"),]),
    );
    m.insert(
        "stash",
        lex!(
            "临时保存工作区的修改，以便切换分支",
            [("git", "将未提交的修改暂存起来，恢复干净的工作区"),]
        ),
    );
    m.insert(
        "branch",
        lex!("分支管理", [("git", "列出、创建或删除 Git 分支"),]),
    );
    m.insert(
        "add",
        lex!(
            "添加",
            [
                ("git", "将文件的更改添加到暂存区，准备下次提交"),
                ("apt", "安装新软件包"),
            ]
        ),
    );
    m.insert(
        "clone",
        lex!(
            "克隆：从远程仓库复制完整的项目副本到本地",
            [("git", "从远程仓库克隆完整的 Git 项目"),]
        ),
    );

    // ── docker 子命令 ──
    m.insert("run", lex!("运行", [("docker", "创建并启动一个新容器"),]));
    m.insert(
        "build",
        lex!("构建", [("docker", "根据 Dockerfile 构建镜像"),]),
    );
    m.insert(
        "exec",
        lex!(
            "执行",
            [
                ("docker", "在运行中的容器内执行命令"),
                ("find", "对匹配的文件执行指定命令"),
            ]
        ),
    );
    m.insert(
        "images",
        lex!("镜像列表", [("docker", "列出本地已有的 Docker 镜像"),]),
    );

    // ── systemctl 子命令 ──
    m.insert(
        "start",
        lex!("启动服务", [("systemctl", "启动指定的 systemd 服务"),]),
    );
    m.insert(
        "stop",
        lex!("停止服务", [("systemctl", "停止指定的 systemd 服务"),]),
    );
    m.insert(
        "restart",
        lex!(
            "重启服务",
            [("systemctl", "停止然后重新启动指定的 systemd 服务"),]
        ),
    );
    m.insert(
        "enable",
        lex!("设为开机启动", [("systemctl", "将服务设为开机自动启动"),]),
    );

    // ── 高频选项（通用 + 命令特定） ──
    m.insert(
        "-n",
        lex!(
            "指定数量或行数",
            [
                ("head", "显示前 N 行"),
                ("tail", "显示末尾 N 行"),
                ("xargs", "每次传递 N 个参数给命令"),
                ("grep", "同时显示匹配行的行号"),
                ("sort", "按数值大小排序（而非字典序）"),
            ]
        ),
    );
    m.insert(
        "-r",
        lex!(
            "递归处理或反转",
            [
                ("cp", "递归复制目录及其内容"),
                ("rm", "递归删除目录及其内容"),
                ("grep", "递归搜索目录下的所有文件"),
                ("sort", "反转排序结果（降序）"),
            ]
        ),
    );
    m.insert(
        "-R",
        lex!(
            "递归处理子目录",
            [
                ("chmod", "递归修改目录下所有文件和子目录的权限"),
                ("chown", "递归修改所有者"),
                ("grep", "递归搜索目录"),
                ("ls", "递归列出子目录内容"),
            ]
        ),
    );
    m.insert(
        "-f",
        lex!(
            "强制执行或指定文件",
            [
                ("rm", "强制删除，不提示确认"),
                ("tail", "持续监视文件末尾的新增内容（follow 模式）"),
                ("tar", "指定归档文件名"),
                ("cut", "指定要提取的字段编号"),
                ("docker", "强制执行操作"),
            ]
        ),
    );
    m.insert(
        "-h",
        lex!(
            "以人类可读格式显示（如 KB/MB/GB）",
            [
                ("du", "以 KB/MB/GB 格式显示磁盘用量"),
                ("df", "以 KB/MB/GB 格式显示磁盘空间"),
                ("free", "以 KB/MB/GB 格式显示内存"),
                ("ls", "以 KB/MB/GB 格式显示文件大小"),
                ("sort", "按人类可读的大小排序（如 2K < 1M < 1G）"),
            ]
        ),
    );
    m.insert(
        "-c",
        lex!(
            "计数或创建",
            [
                ("uniq", "在每行前显示该行出现的次数"),
                ("wc", "只显示字节数"),
                ("sort", "检查文件是否已排序"),
                ("tar", "创建新的归档文件"),
            ]
        ),
    );
    m.insert(
        "-i",
        lex!(
            "交互模式或就地修改",
            [
                ("sed", "直接修改文件内容（就地编辑），而非输出到标准输出"),
                ("rm", "删除前逐个确认"),
                ("cp", "覆盖前确认"),
                ("grep", "忽略大小写"),
                ("docker", "交互模式（保持标准输入打开）"),
            ]
        ),
    );
    m.insert(
        "-u",
        lex!(
            "按用户或服务筛选",
            [
                ("journalctl", "只显示指定 systemd 服务的日志"),
                ("sort", "去重：只保留唯一行"),
                ("docker", "指定用户"),
            ]
        ),
    );
    m.insert(
        "-p",
        lex!(
            "创建父目录或指定端口",
            [
                ("mkdir", "递归创建目录（自动创建不存在的父目录）"),
                ("docker", "端口映射（宿主端口:容器端口）"),
                ("ssh", "指定 SSH 端口号"),
                ("rsync", "保留文件权限"),
            ]
        ),
    );
    m.insert(
        "-s",
        lex!(
            "符号链接或静默",
            [
                ("ln", "创建符号链接（软链接）而非硬链接"),
                ("du", "只显示指定目录的总大小（不列出子目录）"),
                ("curl", "静默模式（不显示进度条）"),
            ]
        ),
    );
    m.insert(
        "-l",
        lex!(
            "长格式显示",
            [
                ("ls", "以长格式显示文件详情（权限、所有者、大小、修改时间）"),
                ("wc", "只显示行数"),
                ("docker", "显示详细信息"),
            ]
        ),
    );
    m.insert(
        "-a",
        lex!(
            "显示全部（包括隐藏项）",
            [
                ("ls", "显示所有文件，包括以 . 开头的隐藏文件"),
                ("ps", "显示所有用户的进程"),
            ]
        ),
    );
    m.insert("-S", lex!("按大小排序", [("ls", "按文件大小降序排列"),]));
    m.insert(
        "-t",
        lex!(
            "按时间排序或指定分隔符",
            [
                ("ls", "按修改时间排序"),
                ("docker", "分配伪终端（TTY）"),
                ("ssh", "强制分配伪终端"),
            ]
        ),
    );
    m.insert(
        "-d",
        lex!(
            "只显示目录或指定分隔符",
            [
                ("ls", "只列出目录本身，不列出其内容"),
                ("cut", "指定字段分隔符"),
            ]
        ),
    );
    m.insert(
        "-v",
        lex!(
            "详细输出或反转匹配",
            [
                ("grep", "反转匹配：显示不匹配的行"),
                ("rsync", "显示详细的传输过程"),
                ("docker", "显示版本信息"),
                ("tar", "显示处理的文件列表"),
            ]
        ),
    );
    m.insert(
        "-e",
        lex!(
            "指定脚本或模式",
            [
                ("sed", "指定要执行的编辑命令"),
                ("grep", "指定搜索模式（可多次使用以匹配多个模式）"),
                ("journalctl", "跳转到日志末尾"),
            ]
        ),
    );
    m.insert(
        "-x",
        lex!(
            "解压或限制",
            [
                ("tar", "从归档文件中提取（解压）文件"),
                ("journalctl", "显示额外的日志元数据（如进程 PID、源码行号）"),
                ("chmod", "设置可执行权限"),
            ]
        ),
    );
    m.insert(
        "-z",
        lex!(
            "使用 gzip 压缩",
            [("tar", "在打包/解包时使用 gzip 算法压缩或解压"),]
        ),
    );
    m.insert(
        "-m",
        lex!(
            "指定消息或模式",
            [
                ("git commit", "直接在命令行指定提交消息（不打开编辑器）"),
                ("docker", "指定内存限制"),
            ]
        ),
    );
    m.insert(
        "-I",
        lex!(
            "指定替换字符串",
            [("xargs", "用 {} 或自定义字符串作为参数替换标记"),]
        ),
    );
    m.insert(
        "-9",
        lex!("SIGKILL 信号：强制终止进程（不可被进程捕获或忽略）"),
    );

    // ── 长选项 ──
    m.insert(
        "--since",
        lex!(
            "只显示指定时间之后的内容",
            [
                ("journalctl", "只显示指定时间之后的日志"),
                ("git log", "只显示指定时间之后的提交"),
            ]
        ),
    );
    m.insert(
        "--delete",
        lex!(
            "删除目标中源目录没有的文件",
            [("rsync", "同步时删除目标目录中多余的文件，保持两端一致"),]
        ),
    );
    m.insert(
        "--oneline",
        lex!(
            "单行格式：每个提交只显示一行（哈希 + 消息）",
            [("git log", "每个提交压缩为一行显示"),]
        ),
    );
    m.insert(
        "--graph",
        lex!(
            "用 ASCII 图形显示分支合并历史",
            [("git log", "用图形化方式展示分支和合并关系"),]
        ),
    );
    m.insert(
        "--format",
        lex!(
            "自定义输出格式",
            [("docker", "使用 Go 模板自定义输出格式"),]
        ),
    );
    m.insert(
        "--amend",
        lex!(
            "修改最近一次提交（追加更改或修改消息）",
            [("git commit", "修改最近一次 Git 提交"),]
        ),
    );
    m.insert(
        "--staged",
        lex!(
            "查看已暂存的更改",
            [("git diff", "显示暂存区与最近提交之间的差异"),]
        ),
    );
    m.insert(
        "--no-edit",
        lex!(
            "不修改提交消息（保持原来的消息）",
            [("git commit", "使用 --amend 时不打开编辑器修改消息"),]
        ),
    );
    m.insert("--decorate", lex!("在提交旁显示分支和标签名称"));
    m.insert("--color", lex!("启用彩色输出"));
    m.insert("--prune", lex!("清理不再存在的远程引用"));
    m.insert(
        "--tail",
        lex!(
            "只显示最后若干行",
            [("docker logs", "只显示容器日志的最后 N 行"),]
        ),
    );
    m.insert(
        "--all",
        lex!(
            "显示所有",
            [
                ("git log", "显示所有分支的提交历史"),
                ("docker", "显示所有容器（包括已停止的）"),
                ("ps", "显示所有进程"),
            ]
        ),
    );
    m.insert("--exclude", lex!("排除匹配的文件或目录"));
    m.insert(
        "--preview",
        lex!("预览模式：只显示将要执行的操作，不实际执行"),
    );
    m.insert("--build", lex!("构建相关参数"));
    m.insert("--icons", lex!("显示文件类型图标"));

    // ── find 谓词 ──
    m.insert(
        "-type",
        lex!(
            "按文件类型筛选",
            [(
                "find",
                "按文件系统条目类型筛选（f=普通文件, d=目录, l=符号链接）"
            ),]
        ),
    );
    m.insert(
        "-name",
        lex!(
            "按文件名模式匹配",
            [("find", "按文件名匹配（支持通配符 * ? []）"),]
        ),
    );
    m.insert(
        "-exec",
        lex!(
            "对匹配的文件执行命令",
            [("find", "对每个匹配的文件执行指定的命令，以 ; 或 + 结尾"),]
        ),
    );

    // ── 特殊值 ──
    m.insert(
        "f",
        lex!(
            "普通文件",
            [("find", "find -type 的值：只匹配普通文件（不含目录和链接）"),]
        ),
    );
    m.insert(
        "d",
        lex!("目录", [("find", "find -type 的值：只匹配目录"),]),
    );
    m.insert(
        "644",
        lex!("权限模式：所有者可读写(rw-)，组用户可读(r--)，其他用户可读(r--)"),
    );
    m.insert(
        "755",
        lex!("权限模式：所有者可读写执行(rwx)，组和其他用户可读执行(r-x)"),
    );
    m.insert("600", lex!("权限模式：仅所有者可读写(rw-)，其他人无权限"));
    m.insert(
        "700",
        lex!("权限模式：仅所有者可读写执行(rwx)，其他人无权限"),
    );
    m.insert("+x", lex!("为文件添加可执行权限"));

    // ── 常见选项组合 ──
    m.insert("-la", lex!("组合选项：-l（长格式）+ -a（显示隐藏文件）"));
    m.insert("-lh", lex!("组合选项：-l（长格式）+ -h（人类可读大小）"));
    m.insert(
        "-lhS",
        lex!("组合选项：-l（长格式）+ -h（人类可读大小）+ -S（按大小排序）"),
    );
    m.insert("-lR", lex!("组合选项：-l（长格式）+ -R（递归列出子目录）"));
    m.insert(
        "-czf",
        lex!("tar 组合选项：-c（创建归档）+ -z（gzip 压缩）+ -f（指定文件名）"),
    );
    m.insert(
        "-xzf",
        lex!("tar 组合选项：-x（解压）+ -z（gzip 解压）+ -f（指定文件名）"),
    );
    m.insert(
        "-sh",
        lex!(
            "组合选项：-s（只显示总计）+ -h（人类可读格式）",
            [("du", "显示指定目录的总磁盘用量（人类可读格式）"),]
        ),
    );
    m.insert(
        "-rn",
        lex!(
            "组合选项：-r（反转/降序）+ -n（数值排序）",
            [("sort", "按数值降序排序"),]
        ),
    );
    m.insert(
        "-av",
        lex!(
            "组合选项：-a（归档模式，保留权限/时间等）+ -v（详细输出）",
            [("rsync", "以归档模式同步文件，并显示传输过程"),]
        ),
    );
    m.insert("-fsSL", lex!("curl 组合选项：-f（HTTP 错误时报错）+ -s（静默）+ -S（静默时仍显示错误）+ -L（跟随重定向）"));
    m.insert(
        "-xeu",
        lex!("journalctl 组合选项：-x（显示解释）+ -e（跳到末尾）+ -u（按服务筛选）"),
    );

    // ── awk/sed 特殊语法 ──
    m.insert(
        "-F:",
        lex!("指定字段分隔符为冒号 :（用于解析 /etc/passwd 等冒号分隔的文件）"),
    );

    // ── 常见搜索关键词 ──
    m.insert(
        "ERROR",
        lex!("搜索关键词：匹配包含 ERROR 的行（常用于日志排查）"),
    );
    m.insert(
        "TODO",
        lex!("搜索关键词：匹配包含 TODO 的行（常用于代码审查）"),
    );

    // ── 服务名 ──
    m.insert(
        "nginx",
        lex!(
            "Nginx Web 服务器",
            [
                ("systemctl", "Nginx 服务单元"),
                ("journalctl", "Nginx 服务的日志"),
            ]
        ),
    );
    m.insert(
        "sshd",
        lex!(
            "SSH 守护进程",
            [
                ("systemctl", "SSH 服务单元"),
                ("journalctl", "SSH 服务的日志"),
            ]
        ),
    );

    // ── 常见文件名 ──
    m.insert("app.log", lex!("应用程序日志文件"));
    m.insert("access.log", lex!("HTTP 访问日志文件"));
    m.insert("server.log", lex!("服务器日志文件"));
    m.insert("notes.txt", lex!("文本笔记文件"));
    m.insert("backup.tar.gz", lex!("gzip 压缩的 tar 归档备份文件"));
    m.insert("deploy.sh", lex!("部署脚本"));
    m.insert("README.md", lex!("项目说明文档"));
    m.insert(".env", lex!("环境变量配置文件"));
    m.insert("data.csv", lex!("CSV 格式的数据文件"));

    // ── 变量 ──
    m.insert("$HOME", lex!("环境变量：用户主目录路径"));
    m.insert("$SHELL", lex!("环境变量：当前用户的默认 Shell 路径"));
    m.insert(
        "$1",
        lex!("位置参数：脚本的第一个参数，或 awk 中第一个字段"),
    );
    m.insert("$0", lex!("awk 中表示整行内容，或脚本中表示脚本自身名称"));

    // ── URL ──
    m.insert("https://example.com", lex!("示例 URL（用于教学演示）"));
    m.insert("https://example.com/health", lex!("健康检查端点 URL"));

    // ── 常见 awk/sed 表达式 ──
    m.insert(
        "'{print $1}'",
        lex!("awk 程序：打印每行的第一个字段（按空格/Tab 分割）"),
    );
    m.insert(
        "'NR==1 {print $0}'",
        lex!("awk 程序：只打印第一行（NR=行号，$0=整行）"),
    );
    m.insert(
        "'s/foo/bar/'",
        lex!("sed 替换表达式：将每行中第一个 foo 替换为 bar"),
    );
    m.insert(
        "'s/localhost/127.0.0.1/'",
        lex!("sed 替换表达式：将 localhost 替换为 IP 地址 127.0.0.1"),
    );
    m.insert("'1,5p'", lex!("sed 范围打印：打印第 1 到第 5 行"));
    m.insert("'a-z'", lex!("字符范围：所有小写字母 a 到 z"));
    m.insert("'A-Z'", lex!("字符范围：所有大写字母 A 到 Z"));

    // ── 常见 glob/pattern ──
    m.insert("'*.log'", lex!("通配符模式：匹配所有以 .log 结尾的文件"));
    m.insert("'*.txt'", lex!("通配符模式：匹配所有以 .txt 结尾的文件"));
    m.insert(
        "'[n]ginx'",
        lex!("grep 技巧：匹配 nginx 但排除 grep 自身进程（方括号使 grep 的 pattern 与进程名不同）"),
    );

    // ── 常见引号内容 ──
    m.insert(
        "'hello world'",
        lex!("字面字符串：包含空格的文本（单引号保护不被 Shell 展开）"),
    );
    m.insert("'ERROR'", lex!("搜索模式：匹配文本 ERROR"));
    m.insert("'TODO'", lex!("搜索模式：匹配文本 TODO"));
    m.insert("'main'", lex!("搜索模式：匹配文本 main"));

    // ── kill 信号 ──
    m.insert("1234", lex!("进程 ID（PID）：目标进程的编号"));

    // ── 用户/组 ──
    m.insert(
        "alice:alice",
        lex!("用户名:组名 格式：将所有者改为 alice，所属组改为 alice"),
    );

    // ── rsync 路径 ──
    m.insert(
        "/srv/www/",
        lex!("源目录：末尾的 / 表示同步目录内容而非目录本身"),
    );
    m.insert("/backup/www/", lex!("备份目标目录"));

    // ── docker 常见参数 ──
    m.insert(
        "-it",
        lex!("Docker 组合选项：-i（交互模式）+ -t（分配终端），用于交互式进入容器"),
    );

    // ── 数字参数 ──
    m.insert(
        "20",
        lex!(
            "数量参数",
            [("head", "显示前 20 行"), ("tail", "显示末尾 20 行"),]
        ),
    );
    m.insert(
        "100",
        lex!(
            "数量参数",
            [
                ("journalctl", "显示最近 100 条日志"),
                ("head", "显示前 100 行"),
            ]
        ),
    );
    m.insert(
        "500",
        lex!(
            "HTTP 状态码 500（服务器内部错误）",
            [("grep", "搜索 HTTP 500 错误"),]
        ),
    );
    m.insert(
        "today",
        lex!("时间关键词：今天", [("journalctl", "只显示今天的日志"),]),
    );

    m
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_basic_tokens() {
        assert!(lookup("|", None).is_some());
        assert!(lookup("find", None).is_some());
        assert!(lookup("grep", None).is_some());
        assert!(lookup("nonexistent_token_xyz", None).is_none());
    }

    #[test]
    fn lookup_with_context() {
        let desc_generic = lookup("-f", None).unwrap();
        let desc_tail = lookup("-f", Some("tail")).unwrap();
        let desc_rm = lookup("-f", Some("rm")).unwrap();
        assert_ne!(desc_tail, desc_rm);
        assert_ne!(desc_generic, desc_tail);
    }

    #[test]
    fn lookup_context_falls_back_to_default() {
        // "sort" context not listed for "-f", should get default
        let desc = lookup("-f", Some("sort")).unwrap();
        assert!(desc.contains("强制") || desc.contains("指定"));
    }

    #[test]
    fn all_entries_have_nonempty_desc() {
        let lex = build_lexicon();
        for (token, entry) in &lex {
            assert!(!entry.desc.is_empty(), "empty desc for token: {}", token);
        }
    }
}
