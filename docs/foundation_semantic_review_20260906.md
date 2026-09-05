# 281 个基础用法的逐项语义复核

本清单逐项覆盖 281 组、843 个练习位置。每组先定义核心能力，再说明原用法、参数变化与小任务迁移如何保留该核心；文件名、数量等参数允许变化，关键操作或方向不能替换为相邻命令。机器规则和对应 ID 保存在同名 JSON，审计脚本逐题运行这些规则。

系统章节中的复合能力按章节本身界定，例如“信号与进程控制”包含信号编号查询和定向发送；这类契约明确列出允许操作，不只检查程序名。所有练习独立还原课程快照，命令不会真实执行。

## 1. Shell 配置文件

组：`practice-system-config_files-bashrc`

核心能力：只读检查或加载当前用户的 Bash 启动配置文件。

教学示例：`cat ~/.bashrc | head -20`

- 原用法：`source ~/.bashrc` — 在当前交互Bash加载自己已审核的启动配置。
- 参数变化：`grep -n 'alias' ~/.bashrc` — 带行号检查启动配置中的别名定义。
- 小任务迁移：`sed -n '1,20p' ~/.bashrc` — 只读查看启动配置前二十行以核对环境设置。

## 2. SSH 服务配置

组：`practice-system-config_files-sshd`

核心能力：读取、展开或校验 SSH 服务端配置。

教学示例：`cat /etc/ssh/sshd_config | grep -v '^#' | grep -v '^$'`

- 原用法：`sudo sshd -t` — 校验SSH服务端配置语法和密钥文件可读性。
- 参数变化：`sudo sshd -T` — 展开并读取SSH服务端的有效配置。
- 小任务迁移：`grep -n '^Include' /etc/ssh/sshd_config` — 检查主配置是否包含额外SSH配置片段。

## 3. Nginx Web 服务器配置

组：`practice-system-config_files-nginx`

核心能力：读取 Nginx 站点配置或校验完整代理配置。

教学示例：`nginx -t`

- 原用法：`cat /etc/nginx/sites-available/default | head -30` — 查看默认站点配置前 30 行。
- 参数变化：`sudo nginx -t` — 在任何 reload 或 restart 前，只验证 nginx 配置语法和引用文件。
- 小任务迁移：`sudo nginx -T` — 校验并打印完整 nginx 配置，输出可能含敏感配置应谨慎分享。

## 4. 文件系统挂载表

组：`practice-system-config_files-fstab`

核心能力：读取或验证持久挂载表 fstab 的配置条目。

教学示例：`cat /etc/fstab`

- 原用法：`findmnt --fstab` — 只读取持久挂载表而不是当前运行态挂载。
- 参数变化：`findmnt --verify --verbose` — 校验fstab可解析性和引用，输出诊断而不挂载。
- 小任务迁移：`grep -v '^#' /etc/fstab` — 过滤注释后核对fstab设备、挂载点与选项字段。

## 5. 定时任务配置

组：`practice-system-config_files-crontab`

核心能力：读取或编辑定时任务配置本身，而不是仅检查调度进程。

教学示例：`crontab -l`

- 原用法：`crontab -e` — 编辑当前用户的定时任务配置并注意五字段格式。
- 参数变化：`crontab -l | head -n 10` — 只读检查当前用户配置的前十行。
- 小任务迁移：`ls -l /etc/cron.d/` — 定位系统级定时任务配置文件并核对权限。

## 6. /etc 系统配置目录

组：`practice-system-directory_structure-etc`

核心能力：检查 /etc 下的系统配置文件、目录或类型。

教学示例：`ls /etc/ | head -20`

- 原用法：`ls -la /etc/ssh/` — 查看 SSH 配置目录的详细内容。
- 参数变化：`file /etc/hostname` — 查看 /etc/hostname 的文件类型。
- 小任务迁移：`cat /etc/hostname` — 直接读取 /etc 中保存主机名的配置文件，迁移到系统配置检查任务。

## 7. /var 可变数据目录

组：`practice-system-directory_structure-var`

核心能力：检查 /var 可变数据尤其日志的目录与占用。

教学示例：`ls /var/`

- 原用法：`du -sh /var/log/* | sort -rh | head -5` — 按大小排序显示 /var/log 下最大的 5 个日志文件或目录。
- 参数变化：`ls -lt /var/log/ | head -10` — 按修改时间排序显示最近更新的日志文件。
- 小任务迁移：`ls /var/log` — 列出 /var/log 的日志条目，保留目录清单操作并迁移到日志定位。

## 8. /usr 用户程序目录

组：`practice-system-directory_structure-usr`

核心能力：检查 /usr 的程序、文档或程序链接。

教学示例：`ls /usr/`

- 原用法：`ls /usr/bin/ | wc -l` — 统计 /usr/bin 下的命令数量。
- 参数变化：`ls /usr/share/doc/ | head -10` — 列出 /usr/share/doc 下前 10 个软件包的文档目录。
- 小任务迁移：`readlink -f /usr/bin/python3` — 解析符号链接得到最终的真实路径。

## 9. /home 用户家目录

组：`practice-system-directory_structure-home`

核心能力：检查用户主目录的内容或占用。

教学示例：`ls -la /home/`

- 原用法：`ls -la ~` — 查看当前用户家目录的所有文件（包括隐藏文件）。
- 参数变化：`du -sh /home/*` — 查看每个用户家目录的磁盘使用量。
- 小任务迁移：`ls ~/ops-lab` — 未引用的波浪号在词首展开为当前用户家目录。

## 10. /tmp 临时文件目录

组：`practice-system-directory_structure-tmp`

核心能力：检查 /tmp 的目录类型、权限或临时子目录。

教学示例：`ls -la /tmp/`

- 原用法：`stat /tmp` — 查看 /tmp 目录的详细元信息（注意 sticky bit 权限）。
- 参数变化：`ls -ld /tmp` — 查看 Sticky Bit 目录 — 其他用户执行位显示为 t。
- 小任务迁移：`find /tmp -maxdepth 1 -type d -print` — 只读清点临时目录第一层子目录，不执行清理。

## 11. /dev 设备文件目录

组：`practice-system-directory_structure-dev`

核心能力：检查 /dev 设备文件或从明确设备读取有限数据。

教学示例：`ls /dev/sd*`

- 原用法：`ls -la /dev/null /dev/zero /dev/random` — 查看三个常用特殊设备文件的详细信息。
- 参数变化：`head -c 16 /dev/urandom | od -An -tx1` — 有限读取十六字节随机设备数据并按十六进制显示。
- 小任务迁移：`ls /dev/sd?` — 列出所有单字母的 SCSI 磁盘设备（如 sda、sdb 等）。

## 12. /proc 进程信息伪文件系统

组：`practice-system-directory_structure-proc`

核心能力：只读查询 /proc 伪文件系统中的内核或资源信息。

教学示例：`ls /proc/ | head -20`

- 原用法：`cat /proc/cpuinfo | head -10` — 查看 CPU 信息的前 10 行。
- 参数变化：`cat /proc/meminfo | head -5` — 查看内存使用信息的前 5 行。
- 小任务迁移：`cat /proc/version` — 查看当前 Linux 内核版本信息。

## 13. /sys 系统信息伪文件系统

组：`practice-system-directory_structure-sys`

核心能力：从 /sys 的 sysfs 属性读取硬件或接口状态。

教学示例：`ls /sys/`

- 原用法：`cat /sys/class/net/ens33/address` — 读取 ens33 网卡的 MAC 地址。
- 参数变化：`cat /sys/class/net/ens33/carrier` — 读取网卡载波：1 表示检测到物理连接，0 表示无载波。
- 小任务迁移：`cat /sys/class/net/ens33/operstate` — 读取内核运行状态，不把管理 UP 误认为链路可用。

## 14. rwx 权限基础

组：`practice-system-filesystem_permissions-rwx-basics`

核心能力：通过 ls 长格式或 stat 读取权限位及所有权。

教学示例：`ls -la /home/`

- 原用法：`stat -c '%A %a %U:%G %n' /etc/passwd` — 以数字和符号两种格式显示文件权限及所有者信息。
- 参数变化：`ls -la /usr/bin/passwd` — 查看 passwd 命令的权限（注意 SUID 标志 s）。
- 小任务迁移：`stat ~/ops-lab/report.txt` — 查看实验报告详细元数据。

## 15. chmod 修改权限

组：`practice-system-filesystem_permissions-chmod`

核心能力：用 chmod 的数值或符号模式修改目标权限。

教学示例：`chmod 755 script.sh && ls -la script.sh`

- 原用法：`chmod u+x deploy.sh && ls -la deploy.sh` — 给所有者添加执行权限。
- 参数变化：`chmod go-w config.ini && ls -la config.ini` — 移除组和其他用户的写权限，保护配置文件。
- 小任务迁移：`find docs/ -type f -exec chmod 644 {} +` — 仅把 docs/ 下普通文件设为 644，保留目录原有的搜索和遍历权限。

## 16. chown/chgrp 修改归属

组：`practice-system-filesystem_permissions-chown-chgrp`

核心能力：通过 chown/chgrp 修改文件或目录的用户/组归属。

教学示例：`chown alice:developers project/ && ls -ld project/`

- 原用法：`chown -R www-data:www-data /var/www/` — 递归修改 /var/www/ 的归属为 Web 服务器用户。
- 参数变化：`chgrp docker /usr/local/bin/docker-compose && ls -la /usr/local/bin/docker-compose` — 将 docker-compose 的所属组改为 docker 组。
- 小任务迁移：`chgrp www-data /srv/www/index.html` — 把 index.html 的所属组改为 www-data。

## 17. umask 默认权限掩码

组：`practice-system-filesystem_permissions-umask`

核心能力：观察或局部设置 umask，理解新建对象权限掩码。

教学示例：`umask`

- 原用法：`umask -S` — 用符号形式读取当前实验 shell 掩码所允许的默认权限。
- 参数变化：`( umask 022; umask )` — 在子 shell 演示 022 掩码，父 shell 不变。
- 小任务迁移：`( umask 077; umask )` — 仅在子 shell 中调整并读取掩码，父 shell 不受影响。

## 18. 特殊权限（SUID/SGID/Sticky Bit）

组：`practice-system-filesystem_permissions-special-permissions`

核心能力：检查或设置 SUID、SGID 或 sticky 特殊权限。

教学示例：`ls -la /usr/bin/passwd`

- 原用法：`ls -la /usr/bin/wall` — 查看 SGID 程序 — 组执行位显示为 s。
- 参数变化：`ls -ld /tmp` — 查看 Sticky Bit 目录 — 其他用户执行位显示为 t。
- 小任务迁移：`chmod 4755 program` — 设置 SUID 权限：程序运行时以所有者（通常是 root）身份执行。

## 19. IP 地址与网卡

组：`practice-system-network_basics-ip-address`

核心能力：查看网络接口链路或IP地址，区别于仅查询路由。

教学示例：`ip addr`

- 原用法：`ip addr show ens33` — 读取指定实验网卡的IP地址。
- 参数变化：`ip address show dev lo` — 改变接口对象以检查回环地址。
- 小任务迁移：`ip link show` — 读取接口链路与MTU状态，区别于IP地址本身。

## 20. DNS 解析

组：`practice-system-network_basics-dns`

核心能力：通过 DNS 查询工具读取域名记录。

教学示例：`dig example.com`

- 原用法：`dig +short example.com` — 快速查询 example.com 的解析结果。
- 参数变化：`dig MX gmail.com` — 查询域名的邮件服务器记录。
- 小任务迁移：`nslookup example.com` — 使用 nslookup 查询域名解析。

## 21. 端口与服务

组：`practice-system-network_basics-ports-services`

核心能力：查看监听端口、传输协议及关联服务进程。

教学示例：`ss -tulnp`

- 原用法：`sudo ss -tulnp 'sport = :80'` — 按精确端口显示TCP或UDP监听及服务进程。
- 参数变化：`ss -ltnp` — 只看TCP监听端口与对应进程。
- 小任务迁移：`netstat -tulnp` — 用传统工具读取相同的监听、协议和进程关系。

## 22. 防火墙 (ufw)

组：`practice-system-network_basics-firewall-ufw`

核心能力：检查或按明确端口/来源设置 UFW 过滤规则。

教学示例：`sudo ufw status verbose`

- 原用法：`sudo ufw allow 22/tcp` — 仅在已确认 SSH 管理端口确为 22 时，先允许该入站端口。
- 参数变化：`sudo ufw allow from 192.168.1.0/24 to any port 3306` — 仅允许局域网访问 MySQL 端口。
- 小任务迁移：`sudo ufw deny 23` — 拒绝 Telnet 端口的入站连接。

## 23. 网络诊断

组：`practice-system-network_basics-network-diagnostics`

核心能力：用有限 ICMP 或路径探测定位网络连通问题。

教学示例：`ping -c 4 8.8.8.8`

- 原用法：`ping -c 4 google.com` — 测试域名解析和网络连通性。
- 参数变化：`traceroute google.com` — 追踪数据包到目标的路由路径。
- 小任务迁移：`mtr -r -c 10 google.com` — 综合 ping + traceroute 的网络诊断工具。

## 24. APT 包管理（Debian/Ubuntu）

组：`practice-system-package_management-apt`

核心能力：在 APT 包管理中查询元数据、刷新索引或安装明确包。

教学示例：`sudo apt update`

- 原用法：`sudo apt install nginx -y` — 安装 nginx 软件包（-y 自动确认）。
- 参数变化：`apt search nginx` — 搜索与 nginx 相关的软件包。
- 小任务迁移：`apt show nginx` — 显示软件包的详细信息。

## 25. YUM/DNF 包管理（RHEL/CentOS/Fedora）

组：`practice-system-package_management-yum-dnf`

核心能力：通过 YUM/DNF 明确的包搜索、信息、安装或移除操作管理包。

教学示例：`sudo yum install httpd -y`

- 原用法：`yum search nginx` — 搜索与 nginx 相关的软件包。
- 参数变化：`yum info nginx` — 显示 nginx 软件包的详细信息。
- 小任务迁移：`sudo yum remove httpd` — 卸载 Apache HTTP 服务器。

## 26. Pacman 包管理（Arch Linux）

组：`practice-system-package_management-pacman`

核心能力：通过 Pacman 的搜索、查询、安装或删除事务管理包。

教学示例：`sudo pacman -S nginx`

- 原用法：`pacman -Ss nginx` — 在仓库中搜索 nginx 相关的包。
- 参数变化：`pacman -Qi nginx` — 查询已安装的 nginx 包详细信息。
- 小任务迁移：`sudo pacman -R nginx` — 卸载 nginx 软件包。

## 27. 常见包管理任务

组：`practice-system-package_management-common-tasks`

核心能力：围绕包文件、依赖与升级计划完成包管理检查。

教学示例：`which nginx`

- 原用法：`dpkg -L nginx | head -10` — 列出某个已安装包包含的所有文件（前 10 条）。
- 参数变化：`apt-cache depends nginx` — 显示软件包的依赖关系。
- 小任务迁移：`sudo apt upgrade` — 审查方案后升级已安装软件包，不自动移除现有包。

## 28. 进程基础

组：`practice-system-process_systemd-process-basics`

核心能力：读取进程清单、父子关系及资源字段。

教学示例：`ps aux | head -10`

- 原用法：`ps aux --sort=-%mem | head -10` — 按内存占用降序排列进程（前 10 行）。
- 参数变化：`pstree -p | head -20` — 以树形结构显示进程及其 PID。
- 小任务迁移：`ps -ef | head -10` — 以完整格式显示所有进程（包含 PPID）。

## 29. 信号与进程控制

组：`practice-system-process_systemd-process-signals`

核心能力：查询信号名称或向已确认练习PID发送明确的控制信号。

教学示例：`kill -l`

- 原用法：`kill -TERM 4242` — 向已确认练习PID发送正常终止请求。
- 参数变化：`kill -INT 4242` — 改变为交互中断信号并理解程序处理的差异。
- 小任务迁移：`kill -l TERM` — 只查询TERM信号对应编号，不发送任何信号。

## 30. 作业控制

组：`practice-system-process_systemd-job-control`

核心能力：操作当前 shell 作业或用 nohup 保持后台任务。

教学示例：`jobs`

- 原用法：`bg %1` — 让编号为 1 的挂起作业转到后台继续运行。
- 参数变化：`fg %1` — 把编号为 1 的后台作业切回前台。
- 小任务迁移：`nohup ./server.sh &` — 在后台启动 server.sh，忽略挂断信号（终端关闭后继续运行）。

## 31. Systemd 服务单元

组：`practice-system-process_systemd-systemd-units`

核心能力：查询或显式启动/停止指定 systemd 服务单元。

教学示例：`systemctl list-units --type=service --state=running | head -15`

- 原用法：`systemctl status nginx` — 查看 nginx 服务当前状态。
- 参数变化：`sudo systemctl start nginx` — 启动 nginx 服务。
- 小任务迁移：`sudo systemctl stop nginx` — 停止 nginx 服务。

## 32. Journalctl 日志查看

组：`practice-system-process_systemd-journalctl`

核心能力：按单元、启动或错误等级读取 journal 记录。

教学示例：`journalctl -u nginx -n 20`

- 原用法：`journalctl -u sshd --since "1 hour ago"` — 按SSH服务与时间窗口读取日志。
- 参数变化：`journalctl -p err -b` — 读取当前启动中的错误级别日志。
- 小任务迁移：`journalctl -u nginx.service -n 10 --no-pager` — 取Nginx最近十条日志进行服务故障检查。

## 33. PID、PPID、SID 与 TTY：先确认进程属于哪个会话

组：`practice-system-terminal_shell_tty-process-session-tty-model`

核心能力：读取进程会话及TTY关联，或检查标准输入的终端属性。

教学示例：`tty`

- 原用法：`ps ww -o user,pid,ppid,sid,tty,stat,args -p 4242` — 按 PID 宽幅查看所有者、父进程、会话 ID、TTY、状态与完整命令行。
- 参数变化：`ps ww -o pid,ppid,sid,pgid,tpgid,tty,stat,args --forest -p 4242` — 以宽幅静态示例查看目标进程的父进程、会话、进程组、前台进程组和完整参数。
- 小任务迁移：`tty -s` — 静默检查标准输入是否为终端，使用退出状态判断。

## 34. tmux：让程序依附于可分离的伪终端

组：`practice-system-terminal_shell_tty-tmux-detach-reattach`

核心能力：使用 tmux 创建、分离或重新连接命名会话。

教学示例：`tmux new -s dev`

- 原用法：`tmux new -s ops` — 创建命名会话并进入独立的伪终端。
- 参数变化：`tmux detach-client -s ops` — 让课程已存在的ops会话客户端分离，保留其中任务。
- 小任务迁移：`tmux attach -t ops` — 重新连接课程已存在的ops会话恢复交互。

## 35. reptyr：仅在身份、所有权与内核策略都验证后考虑

组：`practice-system-terminal_shell_tty-reptyr-safety-boundaries`

核心能力：针对已有进程检查ptrace策略、终端归属或进行已授权接管。

教学示例：`command -v reptyr`

- 原用法：`cat /proc/sys/kernel/yama/ptrace_scope` — 只读查看 Yama ptrace_scope；值 1 仍需结合后代关系、显式授权和能力判断。
- 参数变化：`reptyr 4242` — 在明确排除 Yama 例外的静态场景中尝试普通接管，并预期权限拒绝。
- 小任务迁移：`ps -t pts/7 -o user,pid,ppid,sid,tty,stat,cmd` — 在任何整条 TTY 操作前，只读盘点 pts/7 上的所有者与进程范围。

## 36. 七步场景：SSH 断开后恢复长期任务

组：`practice-system-terminal_shell_tty-detached-task-recovery-scenario`

核心能力：通过tmux会话/窗格身份定位并重新连接长期任务。

教学示例：`tty`

- 原用法：`tmux list-sessions` — 只读列出当前用户的 tmux 会话，先核对名称、窗口数与连接状态。
- 参数变化：`tmux list-panes -t dev:0 -F '#{pane_id} #{pane_pid} #{pane_tty} #{pane_current_command}'` — 按固定字段查看 dev:0 各窗格的 ID、PID、TTY 与当前命令。
- 小任务迁移：`tmux attach -t dev` — 重新连接到名为 dev 的 tmux 会话。

## 37. stdin、stdout、stderr 与文件描述符

组：`practice-system-standard_streams_exit_status-three-standard-streams`

核心能力：显式连接标准输入、标准输出或标准错误到文件。

教学示例：`sort < names.txt`

- 原用法：`sort < ~/ops-lab/hosts.txt` — 把文件接入排序程序的标准输入。
- 参数变化：`printf 'mode=development\n' > ~/ops-lab/app.conf` — 将标准输出覆盖写入配置文件。
- 小任务迁移：`python3 ~/ops-lab/check.py 2> ~/ops-lab/python-errors.txt` — 单独保存标准错误，保持标准输出的原目标。

## 38. 重定向从左到右，管道默认只接 stdout

组：`practice-system-standard_streams_exit_status-redirection-pipeline-order`

核心能力：比较标准错误复制、重定向顺序和stdout管道计数。

教学示例：`umask 077 ↵ practice_dir=$(mktemp -d) ↵ chmod 700 "$practice_dir" ↵ cd "$practice_dir" ↵ ls missing > listing.log 2>&1`

- 原用法：`umask 077 ↵ practice_dir=$(mktemp -d) ↵ chmod 700 "$practice_dir" ↵ cd "$practice_dir" ↵ ls missing 2>&1 > listing.log` — 在私有夹具中演示相反顺序让错误留在终端；逐条回车，确认成功再继续。
- 参数变化：`ls /srv/missing | wc -l` — 管道只传标准输出，错误仍直达终端。
- 小任务迁移：`ls /srv/missing 2>&1 | wc -l` — 先合并错误输出，再通过管道统计。

## 39. 退出状态：0 表示成功，非 0 表示未完成预期

组：`practice-system-standard_streams_exit_status-exit-status-and-short-circuit`

核心能力：及时读取 $? 以区分成功、失败及错误返回。

教学示例：`echo $?`

- 原用法：`true; printf '%s\n' "$?"` — 观察成功命令退出状态。
- 参数变化：`false; printf '%s\n' "$?"` — 观察失败命令退出状态。
- 小任务迁移：`printf 'exit=%s\n' "$?"` — 读取最近命令的退出状态，后续命令会更新该状态。

## 40. 六步场景：命令有报错但统计结果仍是 0

组：`practice-system-standard_streams_exit_status-stream-diagnosis-scenario`

核心能力：针对缺失路径诊断错误流、退出状态和管道统计之间的区别。

教学示例：`ls /srv/missing`

- 原用法：`ls /srv/missing | wc -l` — 管道只传标准输出，错误仍直达终端。
- 参数变化：`ls /srv/missing 2>/dev/null; printf 'exit=%s\n' "$?"` — 隐藏错误文本后仍用退出状态判断前一步是否失败。
- 小任务迁移：`ls /srv/missing 2>&1 | wc -l` — 先合并错误输出，再通过管道统计。

## 41. 块设备、分区与文件系统不是同一层

组：`practice-system-storage_filesystems_mounts-block-device-filesystem-layer`

核心能力：使用lsblk或blkid读取块设备层级与文件系统标识。

教学示例：`lsblk -o NAME,SIZE,TYPE,MOUNTPOINTS`

- 原用法：`lsblk -f` — 查看块设备的文件系统标识。
- 参数变化：`blkid` — 只读识别块设备的文件系统标识。
- 小任务迁移：`lsblk -o NAME,TYPE,SIZE` — 先关联设备名称、类型与大小。

## 42. 挂载点把文件系统接入统一目录树

组：`practice-system-storage_filesystems_mounts-mount-tree-and-namespace`

核心能力：用findmnt把目录路径关联到挂载点与底层文件系统。

教学示例：`findmnt /`

- 原用法：`findmnt -T /var/log` — 查明路径所在的文件系统。
- 参数变化：`findmnt -o TARGET,SOURCE,FSTYPE,OPTIONS /` — 观察根挂载的目标、来源、文件系统与选项。
- 小任务迁移：`findmnt -T /srv/app` — 定位实验部署目录使用的文件系统。

## 43. df 看文件系统，du 看目录树；inode 也可能耗尽

组：`practice-system-storage_filesystems_mounts-capacity-inodes-and-usage`

核心能力：分别读取文件系统inode和目录磁盘占用以区分资源维度。

教学示例：`df -h /`

- 原用法：`df -i /` — 检查根文件系统的索引节点余量。
- 参数变化：`du -sh ~/Downloads` — 汇总下载目录实际占用空间。
- 小任务迁移：`du -h --max-depth=1 ~/projects` — 比较项目目录下一级子目录的占用。

## 44. 七步场景：应用报告 No space left on device

组：`practice-system-storage_filesystems_mounts-disk-full-diagnosis-scenario`

核心能力：按字节容量、inode及目录占用检查No space故障。

教学示例：`df -h /`

- 原用法：`df -h /var/log` — 先检查出错路径所在文件系统的字节余量。
- 参数变化：`df -i /var/log` — 在同一路径核对inode余量，区别于字节容量。
- 小任务迁移：`du -sh /var/log` — 汇总日志目录占用，为定位大目录提供证据。

## 45. 真实身份、有效身份与补充组

组：`practice-system-users_groups_sudo_sessions-identity-and-group-membership`

核心能力：查询账户和组数据库，理解用户身份与组成员来源。

教学示例：`id training`

- 原用法：`groups training` — 查看教学账号所属的全部用户组。
- 参数变化：`getent passwd training` — 只读确认 training 账号是否存在及其登录信息。
- 小任务迁移：`getent group training` — 只读查看 training 主组的数据库记录。

## 46. sudo 授权边界（各题独立，管理查询使用管理员账号）

组：`practice-system-users_groups_sudo_sessions-sudo-authorization-boundary`

核心能力：用sudo列表操作只读审查当前或指定账号的命令授权。

教学示例：`sudo -l`

- 原用法：`sudo -l -U training` — 在已授权管理员身份下只读审查training账号的命令规则。
- 参数变化：`sudo -l -U ace` — 改变被审查账号，仍仅列出sudo授权规则。
- 小任务迁移：`sudo -ll` — 以详细列表检查当前账号的sudo规则与选项。

## 47. 账户存在、用户已登录、进程正在运行是三件事

组：`practice-system-users_groups_sudo_sessions-login-sessions-and-process-context`

核心能力：分别检查登录会话、登录历史及TTY中的进程。

教学示例：`who`

- 原用法：`w` — 查看当前登录的用户和他们在做什么。
- 参数变化：`last -10` — 查看最近 10 条用户登录记录。
- 小任务迁移：`ps -o user,pid,ppid,tty,lstart,args -t pts/2` — 按终端观察 training 会话中的静态进程树线索。

## 48. 七步场景：training 用户无法读取文件且 sudo 被拒绝

组：`practice-system-users_groups_sudo_sessions-access-denied-scenario`

核心能力：围绕同一访问拒绝问题核对账户、路径权限与sudo授权。

教学示例：`id training`

- 原用法：`getent passwd training` — 核对被拒绝用户的NSS身份和登录信息。
- 参数变化：`namei -l /srv/app/settings.ini` — 逐级检查课程目标配置路径的目录遍历和文件权限。
- 小任务迁移：`sudo -l -U training` — 由已授权管理员核对training的sudo命令边界，区别于文件访问位。

## 49. LANG、LC_* 与 LC_ALL 的覆盖关系

组：`practice-system-locale_time_encoding-locale-precedence`

核心能力：读取或临时设置LANG、分类变量和LC_ALL来观察覆盖关系。

教学示例：`locale`

- 原用法：`printenv LANG LC_ALL LC_CTYPE` — 并列读取默认、总覆盖与字符分类变量。
- 参数变化：`LC_ALL=C locale` — 临时使用最高优先级变量观察所有分类的覆盖结果。
- 小任务迁移：`LANG=C.UTF-8 locale` — 只设置默认值并检查已有分类或LC_ALL是否继续覆盖它。

## 50. UTF-8 是编码，不是语言

组：`practice-system-locale_time_encoding-encoding-and-text-bytes`

核心能力：查看编码线索、原始字节或将明确源编码转换为UTF-8。

教学示例：`locale charmap`

- 原用法：`file -bi report.txt` — 观察工具对文本 MIME 类型和字符集的静态判断。
- 参数变化：`printf '中\n' | od -An -tx1` — 观察 UTF-8 字符对应的字节序列。
- 小任务迁移：`iconv -f GB18030 -t UTF-8 legacy.txt` — 把假定为 GB18030 的静态输入转换到 stdout 供预览。

## 51. 时间点、时区、系统时钟与显示格式

组：`practice-system-locale_time_encoding-time-zone-and-clock`

核心能力：以明确日期字段、UTC或时区缩写解释时间显示。

教学示例：`date`

- 原用法：`date -u` — 只读显示当前 UTC 日期和时间。
- 参数变化：`date +%F` — 以 YYYY-MM-DD 形式显示今天日期。
- 小任务迁移：`date +%Z` — 只读显示当前系统时区缩写。

## 52. 七步场景：中文日志乱码且时间相差八小时

组：`practice-system-locale_time_encoding-garbled-time-scenario`

核心能力：同时检查日志编码线索与明确时区显示，区分乱码和八小时时差。

教学示例：`locale`

- 原用法：`locale charmap` — 先确认当前环境解释文本时使用的字符映射。
- 参数变化：`file -bi app.log` — 查看课程日志文件的MIME与编码线索。
- 小任务迁移：`date -u '+%F %T %Z'` — 以明确UTC时间对照本地显示，判断八小时时差是否来自时区。

## 53. 提取日志文件中每行的第一个字段（IP 地址）

组：`practice-lesson-awk-01`

核心能力：使用 awk 的 print $N 按字段提取内容，并可改变字段和分隔符。

教学示例：`awk '{print $1}' access.log`

- 原用法：`awk '{print $2}' access.log` — 从实验日志读取第二列，先确认字段分隔格式。
- 参数变化：`awk '{print $1, $3}' access.log` — 同时读取实验日志第一与第三列。
- 小任务迁移：`awk -F: '{print $1}' /etc/group` — 保留按字段提取的核心，用 -F: 适应组数据库的冒号分隔格式。

## 54. 查看主机名文件

组：`practice-lesson-cat-01`

核心能力：用 cat 直接读取文本文件；可加行号帮助定位。

教学示例：`cat /etc/hostname`

- 原用法：`cat /etc/os-release` — 读取系统发行版标识文件。
- 参数变化：`cat /etc/shells` — 读取系统允许的登录 shell 列表。
- 小任务迁移：`cat -n ~/ops-lab/settings.ini` — 带行号读取实验配置，方便交流具体位置。

## 55. 查看用户的 bash 配置文件

组：`practice-lesson-cat-02`

核心能力：用 cat 直接读取文本文件；可加行号帮助定位。

教学示例：`cat ~/.bashrc`

- 原用法：`cat /etc/os-release` — 读取系统发行版标识文件。
- 参数变化：`cat /etc/shells` — 读取系统允许的登录 shell 列表。
- 小任务迁移：`cat -n ~/ops-lab/settings.ini` — 带行号读取实验配置，方便交流具体位置。

## 56. 回到主目录

组：`practice-lesson-cd-01`

核心能力：把当前 shell 工作目录切回当前用户的主目录。

教学示例：`cd ~`

- 原用法：`cd "$HOME"` — 用已定义 HOME 路径回到自己的主目录。
- 参数变化：`cd` — 省略路径参数，回到当前用户 HOME 所指主目录。
- 小任务迁移：`cd ~ && pwd` — 回到自己的主目录后显示实际路径供核对。

## 57. 切换到上一级目录

组：`practice-lesson-cd-02`

核心能力：使用 .. 相对路径上移目录，保留当前 shell 的目录变化。

教学示例：`cd ..`

- 原用法：`cd ../` — 使用带尾部斜杠的 .. 路径切换到上一级目录。
- 参数变化：`cd ../..` — 连续使用两个 .. 从当前目录上移两级。
- 小任务迁移：`cd .. && pwd` — 上移一级后显示目录，确认相对路径的结果。

## 58. 给脚本设置可执行权限（所有者 rwx，其他人 rx）

组：`practice-lesson-chmod-01`

核心能力：设置所有者 rwx、组和其他人 rx 的 755 权限。

教学示例：`chmod 755 deploy.sh`

- 原用法：`chmod 755 ~/ops-lab/check.sh` — 设置练习脚本为所有者 rwx、组与其他人 rx。
- 参数变化：`chmod -v 755 ~/ops-lab/deploy.sh` — 以详细输出确认另一脚本的 755 执行权限。
- 小任务迁移：`chmod u=rwx,go=rx ~/ops-lab/start.sh` — 用等价符号权限表达 755，使启动脚本仅所有者可写。

## 59. 用符号模式给文件所有者添加执行权限

组：`practice-lesson-chmod-02`

核心能力：用符号模式给文件所有者增加执行权限。

教学示例：`chmod u+x script.sh`

- 原用法：`chmod u+x ~/ops-lab/check.sh` — 只给练习脚本所有者增加执行位。
- 参数变化：`chmod -v u+x ~/ops-lab/deploy.sh` — 给所有者增加执行位并显示变更。
- 小任务迁移：`chmod u+rx ~/ops-lab/start.sh` — 为本人补齐读取和执行脚本所需权限。

## 60. 将 file.txt 复制为 backup.txt

组：`practice-lesson-cp-01`

核心能力：用 cp 复制文件并保留源文件。

教学示例：`cp file.txt backup.txt`

- 原用法：`cp ~/ops-lab/input/report.txt ~/ops-lab/backup/report.txt` — 把实验报告复制到已准备的备份目录。
- 参数变化：`cp -v ~/ops-lab/input/settings.ini ~/ops-lab/backup/` — 复制实验配置并显示源和目标。
- 小任务迁移：`cp -i ~/ops-lab/input/notes.txt ~/ops-lab/backup/notes.txt` — 复制实验笔记，覆盖目标前请求确认。

## 61. 复制文件到 Documents 目录并显示详情

组：`practice-lesson-cp-02`

核心能力：用 cp 的 verbose 选项显示本次复制的源与目标。

教学示例：`cp -v report.pdf ~/Documents/`

- 原用法：`cp -v ~/ops-lab/input/settings.ini ~/ops-lab/backup/` — 复制配置到备份目录并显示源与目标。
- 参数变化：`cp -iv ~/ops-lab/input/notes.txt ~/ops-lab/backup/notes.txt` — 保留详细输出并在覆盖已有笔记备份前询问。
- 小任务迁移：`cp -v ~/ops-lab/input/report.txt ~/ops-lab/backup/report.txt` — 复制另一份报告并用输出核对准确目标路径。

## 62. 发送简单的 GET 请求

组：`practice-lesson-curl-01`

核心能力：发起并读取 HTTP GET 响应正文。

教学示例：`curl https://api.github.com/zen`

- 原用法：`curl https://example.com/` — 发起简单 HTTPS GET 请求并输出正文。
- 参数变化：`curl --connect-timeout 2 --max-time 5 https://example.com/` — 分别限制连接阶段和整个请求时长。
- 小任务迁移：`curl --max-time 5 http://127.0.0.1:8000/health` — 给本机实验健康检查设置总时限。

## 63. 列出所有正在运行的容器

组：`practice-lesson-docker-01`

核心能力：列出运行中的容器及其状态，不把已停止容器混入运行列表。

教学示例：`docker ps`

- 原用法：`docker ps --no-trunc` — 只列运行容器，并显示完整容器ID及启动命令。
- 参数变化：`docker ps --filter status=running` — 显式筛选运行状态的容器。
- 小任务迁移：`docker ps --format '{{.Names}}\t{{.Status}}'` — 用精简字段核对运行容器名称和状态。

## 64. 列出本地所有 Docker 镜像

组：`practice-lesson-docker-02`

核心能力：列出本地镜像，支持名称筛选或输出字段变化。

教学示例：`docker images`

- 原用法：`docker images nginx` — 列出本地 nginx 仓库对应镜像。
- 参数变化：`docker images --digests` — 列出本地镜像时附带内容摘要。
- 小任务迁移：`docker images --format '{{.Repository}}:{{.Tag}}'` — 把本地镜像清单转为仓库加标签形式。

## 65. 在主目录下搜索所有 PDF 文件

组：`practice-lesson-find-01`

核心能力：使用 find 的名称匹配筛选普通文件。

教学示例：`find /home/alice -name '*.pdf'`

- 原用法：`find ~/ops-lab -type f -name '*.txt'` — 在实验目录查找 txt 普通文件，引号阻止 shell 提前展开。
- 参数变化：`find ~/ops-lab -type f -iname '*.LOG'` — 忽略大小写查找实验日志扩展名。
- 小任务迁移：`find ~/ops-lab -maxdepth 2 -type f -name '*.pdf'` — 在实验目录两层以内按名称模式查找 PDF 普通文件，引号防止 shell 提前展开。

## 66. 查看当前仓库的工作区和暂存区状态

组：`practice-lesson-git-01`

核心能力：使用 git status 同时检查工作区和暂存区状态。

教学示例：`git status`

- 原用法：`git status --short` — 同时用两列状态查看暂存区与工作区变化。
- 参数变化：`git status --short --branch` — 在简洁状态中增加当前分支信息。
- 小任务迁移：`git status --porcelain=v1` — 以稳定格式读取仓库变更状态供脚本使用。

## 67. 查看最近5条提交历史（简洁格式）

组：`practice-lesson-git-02`

核心能力：用 git log --oneline 阅读简洁提交历史。

教学示例：`git log --oneline -5`

- 原用法：`git log --oneline -3` — 阅读最近三条提交的一行摘要。
- 参数变化：`git log --oneline -8` — 扩大历史观察窗口到八条提交。
- 小任务迁移：`git log --oneline -- compose.yaml` — 只观察部署配置文件的提交历史。

## 68. 在系统日志中搜索包含 error 的行

组：`practice-lesson-grep-01`

核心能力：使用 grep 按文本或模式筛选匹配行。

教学示例：`grep 'error' /var/log/syslog`

- 原用法：`grep 'warning' backend.log` — 筛选实验后端日志中的小写 warning 行。
- 参数变化：`grep -n 'ERROR' backend.log` — 定位后端 ERROR 记录和行号。
- 小任务迁移：`grep -i 'timeout' backend.log` — 忽略大小写筛选实验超时记录。

## 69. 查看 /etc/passwd 文件的前 10 行（默认行数）

组：`practice-lesson-head-01`

核心能力：用 head -n 读取文件开头指定行数。

教学示例：`head /etc/passwd`

- 原用法：`head -n 3 /etc/passwd` — 用 head -n 限定账户文件前三行，保留只读截取开头的操作。
- 参数变化：`head -n 8 ~/ops-lab/input/report.txt` — 把开头行数改为八行，用于检查报告格式和首段内容。
- 小任务迁移：`head -n 1 ~/ops-lab/input/report.csv` — 先查看实验 CSV 表头再分析字段。

## 70. 查看 README 文件的前 5 行

组：`practice-lesson-head-02`

核心能力：用 head -n 读取文件开头指定行数。

教学示例：`head -n 5 README.md`

- 原用法：`head -n 3 /etc/passwd` — 用 head -n 限定账户文件前三行，保留只读截取开头的操作。
- 参数变化：`head -n 8 ~/ops-lab/input/report.txt` — 把开头行数改为八行，用于检查报告格式和首段内容。
- 小任务迁移：`head -n 1 ~/ops-lab/input/report.csv` — 先查看实验 CSV 表头再分析字段。

## 71. 显示所有网络接口的 IP 地址信息

组：`practice-lesson-ip-01`

核心能力：通过 ip address 查询接口的 IPv4/IPv6 地址。

教学示例：`ip addr show`

- 原用法：`ip -br address show dev ens33` — 以简表查看实验网卡地址。
- 参数变化：`ip -4 address show dev ens33` — 仅检查实验网卡 IPv4 地址。
- 小任务迁移：`ip -6 address show dev ens33` — 检查实验网卡 IPv6 地址及 tentative、dadfailed 等标志。

## 72. 以简洁格式显示所有网络接口和 IP 地址

组：`practice-lesson-ip-02`

核心能力：使用 ip 的 -br 简洁格式读取接口地址。

教学示例：`ip -br addr show`

- 原用法：`ip -br address show dev ens33` — 简洁显示指定实验接口的地址。
- 参数变化：`ip -br -4 address show` — 以每接口一行格式只显示 IPv4 地址。
- 小任务迁移：`ip -br -6 address show` — 用同样的简洁格式核对 IPv6 地址。

## 73. 向进程 12345 发送默认的 SIGTERM 信号

组：`practice-lesson-kill-01`

核心能力：向已确认的练习进程发送 SIGTERM 请求正常终止。

教学示例：`kill 12345`

- 原用法：`kill -TERM 4242` — 向课程已确认归属的进程4242发送正常终止信号。
- 参数变化：`kill -s TERM 5252` — 用显式信号名称语法请求练习进程5252正常终止。
- 小任务迁移：`kill -n 15 6262` — 使用Linux中SIGTERM的编号15请求练习任务6262正常清理退出。

## 74. 显式请求进程 12345 正常退出

组：`practice-lesson-kill-02`

核心能力：向已确认的练习进程发送 SIGTERM 请求正常终止。

教学示例：`kill -TERM 12345`

- 原用法：`kill -TERM 4242` — 向课程已确认归属的进程4242发送正常终止信号。
- 参数变化：`kill -s TERM 5252` — 用显式信号名称语法请求练习进程5252正常终止。
- 小任务迁移：`kill -n 15 6262` — 使用Linux中SIGTERM的编号15请求练习任务6262正常清理退出。

## 75. 列出当前目录的文件

组：`practice-lesson-ls-01`

核心能力：用 ls 列出目录内容，可改变展示字段和隐藏文件范围。

教学示例：`ls`

- 原用法：`ls /var/log` — 列出 /var/log 的日志条目，保留目录清单操作并迁移到日志定位。
- 参数变化：`ls -l ~/ops-lab` — 长格式查看实验目录权限与元数据。
- 小任务迁移：`ls -a ~/ops-lab` — 包含隐藏文件地列出实验目录。

## 76. 列出 /etc 目录的内容

组：`practice-lesson-ls-02`

核心能力：用 ls 列出目录内容，可改变展示字段和隐藏文件范围。

教学示例：`ls /etc`

- 原用法：`ls /var/log` — 列出 /var/log 的日志条目，保留目录清单操作并迁移到日志定位。
- 参数变化：`ls -l ~/ops-lab` — 长格式查看实验目录权限与元数据。
- 小任务迁移：`ls -a ~/ops-lab` — 包含隐藏文件地列出实验目录。

## 77. 在当前目录创建 projects 目录

组：`practice-lesson-mkdir-01`

核心能力：用 mkdir 创建练习目录，原目录内容不被删除。

教学示例：`mkdir projects`

- 原用法：`mkdir ~/ops-lab/reports` — 在已存在的实验目录内创建 reports 子目录。
- 参数变化：`mkdir -p ~/ops-lab/cache/tmp` — 逐层创建实验缓存目录，父目录已存在也可继续。
- 小任务迁移：`mkdir -p ~/ops-lab/{logs,config}` — 用花括号展开准备实验日志和配置目录。

## 78. 一次性创建多级目录结构

组：`practice-lesson-mkdir-02`

核心能力：用 mkdir -p 一次创建缺失的父子目录。

教学示例：`mkdir -p ~/code/python/scripts`

- 原用法：`mkdir -p ~/ops-lab/work/reports` — 一次补齐工作目录与报告子目录。
- 参数变化：`mkdir -p ~/ops-lab/cache/tmp` — 递归创建缓存的父子目录。
- 小任务迁移：`mkdir -p ~/ops-lab/{logs,config}` — 配合花括号展开准备日志和配置目录。

## 79. 将 old.txt 重命名为 new.txt

组：`practice-lesson-mv-01`

核心能力：用 mv 将文件重命名为另一个明确文件名。

教学示例：`mv old.txt new.txt`

- 原用法：`mv ~/ops-lab/draft.txt ~/ops-lab/final.txt` — 把练习草稿重命名为 final.txt。
- 参数变化：`mv -i ~/ops-lab/notes.old ~/ops-lab/notes.txt` — 在同名目标存在时先确认再重命名。
- 小任务迁移：`mv -v ~/ops-lab/old.log ~/ops-lab/archive.log` — 重命名日志并显示源目标关系。

## 80. 将文件移动到 Documents 目录

组：`practice-lesson-mv-02`

核心能力：用 mv 将文件移入目标目录，目标参数以 / 表示目录。

教学示例：`mv report.pdf ~/Documents/`

- 原用法：`mv ~/ops-lab/report.txt ~/ops-lab/backup/` — 将练习报告移入已有备份目录。
- 参数变化：`mv -i ~/ops-lab/notes.txt ~/ops-lab/backup/` — 移入目录时先确认同名覆盖。
- 小任务迁移：`mv -v ~/ops-lab/old.log ~/ops-lab/archive/` — 将日志移入归档目录并显示移动结果。

## 81. 测试 Nginx 配置文件语法

组：`practice-lesson-nginx-01`

核心能力：用 nginx -t 或 -T 校验配置语法及引用文件。

教学示例：`nginx -t`

- 原用法：`sudo nginx -t` — 以有权读取配置的身份校验Nginx配置。
- 参数变化：`sudo nginx -T` — 在校验配置的同时打印实际加载内容。
- 小任务迁移：`sudo nginx -t -c /etc/nginx/nginx.conf` — 显式指定主配置文件进行语法和引用检查。

## 82. 查看 Nginx 服务运行状态

组：`practice-lesson-nginx-02`

核心能力：用 systemctl status 阅读指定服务状态与最近诊断。

教学示例：`systemctl status nginx`

- 原用法：`systemctl status nginx.service --no-pager` — 查看实验 Web 服务当前状态。
- 参数变化：`systemctl status ssh.service --no-pager` — 查看 Ubuntu SSH 服务状态。
- 小任务迁移：`systemctl status cron.service --no-pager --full` — 完整查看 Ubuntu 定时任务服务状态。

## 83. 向 Google DNS 发送 4 个 ICMP 包测试外网连通性

组：`practice-lesson-ping-01`

核心能力：用有限次数的 ICMP 回显采样连通性与往返时间。

教学示例：`ping -c 4 8.8.8.8`

- 原用法：`ping -c 2 192.0.2.1` — 向实验网关发两次 ICMP 回显请求。
- 参数变化：`ping -c 5 -W 1 192.0.2.1` — 采样五次并限制每次等待，比较丢包情况。
- 小任务迁移：`ping -n -c 3 198.51.100.20` — 直接测试实验 IP 并禁用名称解析，隔离 DNS 干扰。

## 84. 测试域名解析和网络连通性

组：`practice-lesson-ping-02`

核心能力：先解析目标域名，再用有限 ICMP 回显检查对应地址的连通性。

教学示例：`ping -c 3 google.com`

- 原用法：`ping -c 2 api.example.test` — 先解析实验 API 域名，再发送两次 ICMP 请求；回显失败不等于解析失败。
- 参数变化：`ping -c 3 -W 1 db.example.test` — 解析数据库实验域名并进行三次有限等待的连通性采样。
- 小任务迁移：`ping -4 -c 2 example.test` — 使用实验域名解析到的 IPv4 地址进行两次连通性测试。

## 85. 查看所有用户的所有进程（BSD 风格）

组：`practice-lesson-ps-01`

核心能力：使用 BSD 风格 ps aux 查看全部用户进程。

教学示例：`ps aux`

- 原用法：`ps aux --sort=pid` — 以BSD完整格式查看全部用户进程并按PID排序。
- 参数变化：`ps aux --sort=-%mem` — 保留全部进程范围并按内存占用降序排列。
- 小任务迁移：`ps aux --forest` — 用BSD完整格式和进程树关系观察全部进程。

## 86. 查看所有进程的完整信息（POSIX 风格）

组：`practice-lesson-ps-02`

核心能力：使用 POSIX 风格 ps -ef 查看全部进程的完整信息。

教学示例：`ps -ef`

- 原用法：`ps -ef --sort=pid` — 以POSIX完整格式查看全部进程并按PID排序。
- 参数变化：`ps -ef --forest` — 保留全部进程完整信息并显示父子树形关系。
- 小任务迁移：`ps -ef --no-headers` — 去掉表头以便处理完整进程清单。

## 87. 删除单个文件

组：`practice-lesson-rm-01`

核心能力：删除已确认可丢弃的练习普通文件。

教学示例：`rm temp.log`

- 原用法：`rm -v ~/ops-lab/cache.tmp` — 移除已确认的实验临时文件并打印名称。
- 参数变化：`rm -i ~/ops-lab/obsolete.txt` — 交互确认后移除实验过期文件。
- 小任务迁移：`rm -- ~/ops-lab/old-report.txt` — 明确结束选项解析后删除已核对的实验报告。

## 88. 删除文件前确认

组：`practice-lesson-rm-02`

核心能力：用 rm -i 在删除练习文件前逐项询问。

教学示例：`rm -i important.doc`

- 原用法：`rm -i ~/ops-lab/obsolete.txt` — 交互确认后移除实验过期文件。
- 参数变化：`rm -i ~/ops-lab/old-report.txt` — 对另一份实验报告执行交互删除。
- 小任务迁移：`rm -iv ~/ops-lab/cache.tmp` — 确认删除后再显示被处理的实验文件。

## 89. 将日志中所有 error 替换为 ERROR

组：`practice-lesson-sed-01`

核心能力：使用 sed 的 s/旧/新/ 替换表达式转换文本。

教学示例：`sed 's/error/ERROR/g' app.log`

- 原用法：`sed 's/WARN/WARNING/g' backend.log` — 预览替换日志中的 WARN 文本，不修改原文件。
- 参数变化：`sed 's/localhost/127.0.0.1/g' config.example` — 预览文本替换后的配置，不使用原地修改选项。
- 小任务迁移：`sed 's/8080/8000/' config.example` — 预览每行首次端口文本替换，确认后再决定写入。

## 90. 按字母顺序排列文件中的名字

组：`practice-lesson-sort-01`

核心能力：对文本进行默认升序排序，可去重但不改成数值或倒序规则。

教学示例：`sort names.txt`

- 原用法：`sort hosts.txt` — 按当前语言环境升序排列主机名称。
- 参数变化：`sort users.txt` — 将另一份用户名称列表按文本规则升序排列。
- 小任务迁移：`LC_ALL=C sort -u labels.txt` — 使用固定C排序规则整理并去重标签。

## 91. 最基本的 SSH 登录

组：`practice-lesson-ssh-01`

核心能力：以指定远端用户建立到实验 IP 地址的 SSH 会话。

教学示例：`ssh user@192.168.1.100`

- 原用法：`ssh ops@192.0.2.20` — 以 ops 账号连接实验服务器。
- 参数变化：`ssh -p 2222 ops@192.0.2.20` — 使用实验 SSH 非默认端口连接。
- 小任务迁移：`ssh -o ConnectTimeout=5 ops@192.0.2.20` — 限制实验 SSH 连接等待为五秒。

## 92. 使用域名和指定用户登录

组：`practice-lesson-ssh-02`

核心能力：以明确用户连接域名 SSH 主机并要求首次主机身份核验。

教学示例：`ssh root@myserver.com`

- 原用法：`ssh alice@server.example.com` — 首次登录先通过可信渠道核验主机指纹。
- 参数变化：`ssh -p 2222 alice@server.example.com` — 通过 2222 端口登录远程主机。
- 小任务迁移：`ssh -p 2222 admin@example.com` — 通过 2222 端口 SSH 登录远程主机。

## 93. 查看 Nginx 服务的当前运行状态

组：`practice-lesson-systemctl-01`

核心能力：用 systemctl status 阅读指定服务状态与最近诊断。

教学示例：`systemctl status nginx`

- 原用法：`systemctl status nginx.service --no-pager` — 查看实验 Web 服务当前状态。
- 参数变化：`systemctl status ssh.service --no-pager` — 查看 Ubuntu SSH 服务状态。
- 小任务迁移：`systemctl status cron.service --no-pager --full` — 完整查看 Ubuntu 定时任务服务状态。

## 94. 启动 SSH 服务

组：`practice-lesson-systemctl-02`

核心能力：通过 systemctl start 启动指定服务。

教学示例：`systemctl start sshd`

- 原用法：`sudo systemctl start nginx.service` — 启动指定Nginx服务单元。
- 参数变化：`sudo systemctl start cron.service` — 使用相同start操作启动调度服务。
- 小任务迁移：`sudo systemctl start ssh.service` — 启动Ubuntu实验机的SSH服务单元。

## 95. 查看系统日志的最后 10 行

组：`practice-lesson-tail-01`

核心能力：用 tail 读取文件末尾有限行数并退出。

教学示例：`tail /var/log/syslog`

- 原用法：`tail -n 3 backend.log` — 只读取后端日志末尾三行并退出。
- 参数变化：`tail -n 12 backend.log` — 改变行数参数，读取末尾十二行。
- 小任务迁移：`tail -n 5 ~/ops-lab/settings.ini` — 读取配置末尾五行检查最近一段设置。

## 96. 查看最近执行的 5 条命令

组：`practice-lesson-tail-02`

核心能力：用 tail 读取文件末尾有限行数并退出。

教学示例：`tail -n 5 ~/.bash_history`

- 原用法：`tail -n 3 backend.log` — 只读取后端日志末尾三行并退出。
- 参数变化：`tail -n 12 backend.log` — 改变行数参数，读取末尾十二行。
- 小任务迁移：`tail -n 5 ~/ops-lab/settings.ini` — 读取配置末尾五行检查最近一段设置。

## 97. 将 project 目录打包并用 gzip 压缩

组：`practice-lesson-tar-01`

核心能力：创建包含指定目录的 gzip 压缩 tar 归档。

教学示例：`tar -czf project.tar.gz project/`

- 原用法：`tar -czf reports.tar.gz reports` — 将报告目录创建为gzip压缩tar归档。
- 参数变化：`tar -czf settings.tar.gz -C ~/ops-lab config` — 从实验根目录相对归档配置目录并使用gzip。
- 小任务迁移：`tar -czf logs.tar.gz -C ~/ops-lab logs` — 把日志目录归档为gzip包以便保存或传输。

## 98. 解压 .tar.gz 归档文件

组：`practice-lesson-tar-02`

核心能力：将 gzip 压缩的 tar 归档解压到指定目录。

教学示例：`tar -xzf project.tar.gz`

- 原用法：`tar -xzf reports.tar.gz -C ~/ops-lab/output` — 将可信报告gzip归档解压到已有暂存目录。
- 参数变化：`tar -xzf settings.tar.gz -C ~/ops-lab/restore` — 改变归档和目标，保留gzip解压操作。
- 小任务迁移：`tar -xzf logs.tar.gz --no-same-owner -C ~/ops-lab/output` — 解压日志归档时不恢复归档声明的原所有者。

## 99. 启动 top 实时监控系统状态

组：`practice-lesson-top-01`

核心能力：用 top 采样进程资源和系统状态，并控制刷新或观察范围。

教学示例：`top`

- 原用法：`top -b -n 1` — 以批处理模式输出一次进程资源快照。
- 参数变化：`top -b -n 2 -d 1` — 间隔一秒采样两次进程资源。
- 小任务迁移：`top -p 4242` — 只观察已核对的实验进程，退出交互界面使用 q。

## 100. 排序后去除重复的名字

组：`practice-lesson-uniq-01`

核心能力：使用 uniq 合并连续相同的行，必要时先排序。

教学示例：`sort names.txt | uniq`

- 原用法：`uniq sorted-hosts.txt` — 合并实验文件中的相邻重复主机行。
- 参数变化：`uniq -c sorted-hosts.txt` — 统计相邻重复行的出现次数。
- 小任务迁移：`sort hosts.txt | uniq -c` — 先排序再计数，让不相邻的相同主机聚合。

## 101. 确认一个名称在 Bash 中属于哪类命令

组：`practice-lesson-v03_20_command_discovery_rescue-01`

核心能力：使用 Bash type 区分内建、别名、函数和外部命令。

教学示例：`type echo`

- 原用法：`type -a curl` — 显示curl在当前Bash中所有可用定义和外部路径。
- 参数变化：`type -t printf` — 仅输出printf的命令类别。
- 小任务迁移：`type -a cd` — 核对cd属于shell内建而不是普通外部程序。

## 102. 从 PATH 中定位远程登录程序的路径

组：`practice-lesson-v03_20_command_discovery_rescue-02`

核心能力：从 PATH 定位指定外部工具并确认是否安装。

教学示例：`which ssh`

- 原用法：`command -v python3` — 从当前PATH定位Python外部程序。
- 参数变化：`which tar` — 使用which定位归档工具的外部路径。
- 小任务迁移：`command -v curl` — 在运行网络检查前确认curl能否从PATH找到。

## 103. 忘记 ls 选项时查看完整本机手册

组：`practice-lesson-v03_20_help_manuals-01`

核心能力：使用 man 查询命令或配置文件的本机手册。

教学示例：`man ls`

- 原用法：`man cp` — 打开文件复制命令手册，练习 q 退出。
- 参数变化：`man 5 ssh_config` — 查看 SSH 客户端配置文件格式手册。
- 小任务迁移：`man 8 ss` — 查看套接字检查工具管理手册。

## 104. 忘记文本搜索选项时查看简明帮助

组：`practice-lesson-v03_20_help_manuals-02`

核心能力：读取命令行 --help 使用帮助。

教学示例：`grep --help`

- 原用法：`find --help` — 快速查看 GNU find 条件和操作选项。
- 参数变化：`tar --help` — 快速查看 tar 模式与压缩选项。
- 小任务迁移：`curl --help all` — 查看 curl 的完整帮助选项列表。

## 105. 在 Bash 内查看切换目录功能的帮助

组：`practice-lesson-v03_20_help_manuals-03`

核心能力：使用 Bash help 查看 shell 内建命令帮助。

教学示例：`help cd`

- 原用法：`help printf` — 查看 Bash 内建 printf 帮助。
- 参数变化：`help type` — 查看 Bash 命令识别内建帮助。
- 小任务迁移：`help test` — 查看 Bash 条件测试语法帮助。

## 106. 列出当前 Bash 会话的带编号命令历史

组：`practice-lesson-v03_20_shell_history_jobs-01`

核心能力：读取当前 Bash 会话的带编号历史记录。

教学示例：`history`

- 原用法：`history 5` — 查看当前 Bash 会话最近五条历史。
- 参数变化：`history 20` — 扩大历史查看窗口到二十条。
- 小任务迁移：`history | tail -n 8` — 用管道只查看历史末尾八行。

## 107. 只查看最近十条 Bash 命令历史

组：`practice-lesson-v03_20_shell_history_jobs-02`

核心能力：读取当前 Bash 会话的带编号历史记录。

教学示例：`history 10`

- 原用法：`history 5` — 查看当前 Bash 会话最近五条历史。
- 参数变化：`history 20` — 扩大历史查看窗口到二十条。
- 小任务迁移：`history | tail -n 8` — 用管道只查看历史末尾八行。

## 108. 列出当前 Bash 会话中已有的全部别名

组：`practice-lesson-v03_20_shell_history_jobs-03`

核心能力：只读查看当前 Bash 中的别名定义。

教学示例：`alias`

- 原用法：`alias ls` — 只读查看当前会话已定义的ls别名。
- 参数变化：`alias -p` — 按可复用格式列出全部已有别名定义。
- 小任务迁移：`alias ll` — 只读核对课程预置ll别名的实际展开内容。

## 109. 安装或升级前刷新本机的软件包索引

组：`practice-lesson-v03_21_apt_indexes_search-01`

核心能力：刷新 APT 包索引，不以安装、查询或升级代替。

教学示例：`sudo apt-get update`

- 原用法：`sudo apt-get update -o APT::Update::Error-Mode=any` — 任何索引源更新失败都以失败结束，便于发现不完整更新。
- 参数变化：`sudo apt-get -q update` — 刷新实验 Ubuntu 软件源索引并减少进度输出。
- 小任务迁移：`sudo apt-get update -o Acquire::Retries=1` — 刷新索引时为获取过程设置一次重试。

## 110. 按名称和描述搜索 ripgrep

组：`practice-lesson-v03_21_apt_indexes_search-02`

核心能力：按名称或描述搜索 APT 软件包。

教学示例：`apt search ripgrep`

- 原用法：`apt search '^curl$'` — 按精确名称正则搜索curl软件包。
- 参数变化：`apt search tcpdump` — 按名称和描述查找抓包工具。
- 小任务迁移：`apt search openssh-server` — 搜索提供SSH服务端的软件包。

## 111. 从软件源安装 ripgrep

组：`practice-lesson-v03_21_apt_install_upgrade-01`

核心能力：使用 APT install 安装包或预演安装计划。

教学示例：`sudo apt install ripgrep`

- 原用法：`sudo apt install tree` — 在实验 Ubuntu 安装目录树工具，审查提示的依赖变化。
- 参数变化：`sudo apt install jq` — 安装实验 JSON 命令行工具。
- 小任务迁移：`sudo apt-get -s install dnsutils` — 先模拟安装 DNS 工具，确认依赖和空间需求。

## 112. 确认 ripgrep 已安装及其版本

组：`practice-lesson-v03_21_apt_install_upgrade-02`

核心能力：查询指定包是否已安装及安装版本。

教学示例：`apt list --installed ripgrep`

- 原用法：`apt list --installed curl` — 检查 curl 包是否已经安装。
- 参数变化：`apt list --installed openssh-client` — 检查 SSH 客户端包版本。
- 小任务迁移：`dpkg-query -W tcpdump` — 读取已安装 tcpdump 的包名和版本。

## 113. 列出 curl 软件包安装的所有文件

组：`practice-lesson-v03_21_dpkg_repair_cleanup-01`

核心能力：通过 dpkg -L 查看某个已安装软件包的文件清单。

教学示例：`dpkg -L curl`

- 原用法：`dpkg -L tar` — 查看tar软件包安装的文件清单。
- 参数变化：`dpkg -L openssh-client` — 列出SSH客户端包的安装路径。
- 小任务迁移：`dpkg -L bash` — 定位Bash软件包提供的程序和文档文件。

## 114. 只读检查是否有部分安装或状态异常的软件包

组：`practice-lesson-v03_21_dpkg_repair_cleanup-02`

核心能力：通过 dpkg --audit 只读检查包数据库的未完成安装状态。

教学示例：`dpkg --audit`

- 原用法：`dpkg --audit curl` — 审计curl安装状态是否不完整。
- 参数变化：`dpkg --audit nginx` — 针对nginx检查包数据库中的未完成状态。
- 小任务迁移：`dpkg --audit openssh-client` — 在修复SSH客户端前检查其包安装状态。

## 115. 确认归档名不存在后，把 logs 打包为未压缩归档

组：`practice-lesson-v03_22_tar_create_inspect-01`

核心能力：用 tar -c 创建未压缩归档，保留源目录。

教学示例：`tar -cf logs.tar logs`

- 原用法：`tar -cf notes.tar notes` — 将笔记目录创建为未压缩tar归档。
- 参数变化：`tar -cf reports.tar reports` — 改变归档对象，仍保持未压缩格式。
- 小任务迁移：`tar -cf config.tar -C ~/ops-lab config` — 从实验根目录相对创建配置未压缩归档。

## 116. 确认归档名不存在后，把 public 打包为 gzip 归档

组：`practice-lesson-v03_22_tar_create_inspect-02`

核心能力：创建包含指定目录的 gzip 压缩 tar 归档。

教学示例：`tar -czf website.tar.gz public`

- 原用法：`tar -czf reports.tar.gz reports` — 将报告目录创建为gzip压缩tar归档。
- 参数变化：`tar -czf settings.tar.gz -C ~/ops-lab config` — 从实验根目录相对归档配置目录并使用gzip。
- 小任务迁移：`tar -czf logs.tar.gz -C ~/ops-lab logs` — 把日志目录归档为gzip包以便保存或传输。

## 117. 不展开文件，查看 ZIP 内容清单

组：`practice-lesson-v03_22_tar_extract_safety-01`

核心能力：只列出 ZIP 归档的成员而不写出文件。

教学示例：`unzip -l source-code.zip`

- 原用法：`unzip -l reports.zip` — 先查看实验报告压缩包清单。
- 参数变化：`unzip -l config.zip` — 检查配置压缩包成员路径。
- 小任务迁移：`zipinfo -1 site.zip` — 每行只显示一个实验 ZIP 成员路径。

## 118. 不写出文件，检查 ZIP 数据完整性

组：`practice-lesson-v03_22_tar_extract_safety-02`

核心能力：使用 unzip -t 测试 ZIP 数据完整性而不解压。

教学示例：`unzip -t source-code.zip`

- 原用法：`unzip -t reports.zip` — 检查实验报告 ZIP 各成员完整性。
- 参数变化：`unzip -t config.zip` — 校验实验配置 ZIP 是否损坏。
- 小任务迁移：`unzip -t site.zip` — 测试 ZIP 成员完整性，不提取到工作目录。

## 119. 确认压缩包不存在后，把两个文件放入新 ZIP

组：`practice-lesson-v03_22_zip_gzip_xz-01`

核心能力：把明确选择的文件或目录创建为 ZIP 归档。

教学示例：`zip documents.zip report.pdf notes.txt`

- 原用法：`zip notes.zip notes.txt todo.txt` — 将两份实验笔记压入 ZIP。
- 参数变化：`zip -r reports.zip reports` — 递归压缩实验报告目录。
- 小任务迁移：`zip -r config.zip config` — 归档实验配置目录，真实配置可能含凭据应控制备份访问。

## 120. 确认压缩包不存在后，递归压缩 src 目录

组：`practice-lesson-v03_22_zip_gzip_xz-02`

核心能力：使用 zip -r 递归收录整个目录树。

教学示例：`zip -r source-code.zip src`

- 原用法：`zip -r reports.zip reports` — 递归压缩报告目录为ZIP。
- 参数变化：`zip -r config.zip config` — 将配置子目录全部递归收录。
- 小任务迁移：`zip -r logs.zip logs` — 递归打包日志目录供移交使用。

## 121. 自建私有目录树并只查看前两层

组：`practice-lesson-v03_23_deletion_inventory-01`

核心能力：通过 tree -L 限制深度只读查看目录结构。

教学示例：`umask 077 ↵ practice_dir=$(mktemp -d) ↵ chmod 700 "$practice_dir" ↵ cd "$practice_dir" ↵ mkdir -p archive assets/icons backup shared work/docs ↵ touch archive/report.csv assets/logo.png backup/settings.ini settings.ini ↵ ln -s shared current ↵ tree -L 2 .`

- 原用法：`tree -L 1 ~/ops-lab` — 先查看实验目录的第一层结构。
- 参数变化：`tree -L 2 ~/ops-lab/input` — 展开两层实验输入目录。
- 小任务迁移：`tree -a -L 2 ~/ops-lab` — 包含隐藏文件观察实验目录结构。

## 122. 在私有夹具中仅删除刚创建的空目录

组：`practice-lesson-v03_23_deletion_inventory-02`

核心能力：用 rmdir 仅删除空目录，遇到非空目录应失败。

教学示例：`umask 077 ↵ practice_dir=$(mktemp -d) ↵ chmod 700 "$practice_dir" ↵ cd "$practice_dir" ↵ mkdir empty ↵ rmdir empty`

- 原用法：`rmdir ~/ops-lab/empty` — 仅删除空的实验目录，非空时会失败。
- 参数变化：`rmdir -v ~/ops-lab/old-empty` — 删除空目录并打印处理路径。
- 小任务迁移：`rmdir ~/ops-lab/cache/empty` — 清理已确认没有内容的实验缓存子目录。

## 123. 在唯一私有练习目录中一次创建多级目录

组：`practice-lesson-v03_23_file_create_copy_move-01`

核心能力：用 mkdir -p 一次创建缺失的父子目录。

教学示例：`umask 077 ↵ practice_dir=$(mktemp -d) ↵ chmod 700 "$practice_dir" ↵ mkdir -p "$practice_dir/work/docs"`

- 原用法：`mkdir -p ~/ops-lab/work/reports` — 一次补齐工作目录与报告子目录。
- 参数变化：`mkdir -p ~/ops-lab/cache/tmp` — 递归创建缓存的父子目录。
- 小任务迁移：`mkdir -p ~/ops-lab/{logs,config}` — 配合花括号展开准备日志和配置目录。

## 124. 在唯一私有练习目录中创建两个空文件

组：`practice-lesson-v03_23_file_create_copy_move-02`

核心能力：用 touch 创建不存在的空文件或更新文件时间戳。

教学示例：`umask 077 ↵ practice_dir=$(mktemp -d) ↵ chmod 700 "$practice_dir" ↵ touch "$practice_dir/notes.txt" "$practice_dir/todo.txt"`

- 原用法：`touch ~/ops-lab/notes.txt` — 创建空实验笔记，若已存在则更新时间。
- 参数变化：`touch ~/ops-lab/todo.txt ~/ops-lab/done.txt` — 一次准备两份实验文本文件。
- 小任务迁移：`touch -r ~/ops-lab/notes.txt ~/ops-lab/notes.copy` — 将实验副本时间戳对齐到参考文件。

## 125. 在唯一私有练习目录中创建相对符号链接

组：`practice-lesson-v03_23_links_metadata-01`

核心能力：使用 ln -s 创建符号链接而不是复制目标。

教学示例：`umask 077 ↵ practice_dir=$(mktemp -d) ↵ chmod 700 "$practice_dir" ↵ cd "$practice_dir" ↵ mkdir shared ↵ ln -s shared current`

- 原用法：`ln -s releases/v1 ~/ops-lab/current` — 建立指向练习发布目录的相对符号链接。
- 参数变化：`ln -s ../shared ~/ops-lab/input/shared` — 从链接所在input目录解析上一级shared目标。
- 小任务迁移：`ln -s ~/ops-lab/settings.ini ~/ops-lab/settings-link` — 为练习配置文件建立绝对目标符号链接。

## 126. 自建私有夹具并稳定显示链接名称和目标

组：`practice-lesson-v03_23_links_metadata-02`

核心能力：读取符号链接保存的目标文本，并可加标签显示。

教学示例：`umask 077 ↵ practice_dir=$(mktemp -d) ↵ chmod 700 "$practice_dir" ↵ cd "$practice_dir" ↵ mkdir shared ↵ ln -s shared current ↵ printf 'current -> %s\n' "$(readlink current)"`

- 原用法：`readlink ~/ops-lab/current` — 读取current链接保存的目标文本。
- 参数变化：`readlink -n ~/ops-lab/settings-link` — 读取另一链接目标并省略末尾换行。
- 小任务迁移：`printf 'target=%s\n' "$(readlink ~/ops-lab/current)"` — 把链接文本加上标签用于检查记录。

## 127. 在私有夹具中查看报表的精确字节大小

组：`practice-lesson-v03_23_links_metadata-06`

核心能力：使用 stat 的 %s 读取文件的精确逻辑字节长度。

教学示例：`umask 077 ↵ practice_dir=$(mktemp -d) ↵ chmod 700 "$practice_dir" ↵ cd "$practice_dir" ↵ truncate -s 24K report.csv ↵ stat -c '%n %s bytes' report.csv`

- 原用法：`stat -c '%n %s' ~/ops-lab/report.txt` — 只读文件名与字节大小。
- 参数变化：`stat -c '%s' ~/ops-lab/settings.ini` — 只显示实验配置的精确字节数。
- 小任务迁移：`stat -c '%n %s bytes' ~/ops-lab/notes.txt` — 带单位标签读取实验笔记的精确大小。

## 128. 在私有夹具中识别真实脚本类型

组：`practice-lesson-v03_23_links_metadata-07`

核心能力：用 file 按内容识别真实文件格式，不依赖文件扩展名。

教学示例：`umask 077 ↵ practice_dir=$(mktemp -d) ↵ chmod 700 "$practice_dir" ↵ cd "$practice_dir" ↵ printf '#!/bin/sh\nexit 0\n' > run.sh ↵ chmod 700 run.sh ↵ file run.sh`

- 原用法：`file ~/ops-lab/check.sh` — 按内容识别实验脚本类型。
- 参数变化：`file ~/ops-lab/download` — 识别没有扩展名的实验下载文件。
- 小任务迁移：`file -bi ~/ops-lab/report.txt` — 查看实验文本 MIME 类型和猜测的编码。

## 129. 在私有夹具中识别无扩展名文本的实际类型

组：`practice-lesson-v03_23_links_metadata-08`

核心能力：用 file 按内容识别真实文件格式，不依赖文件扩展名。

教学示例：`umask 077 ↵ practice_dir=$(mktemp -d) ↵ chmod 700 "$practice_dir" ↵ cd "$practice_dir" ↵ printf 'sample\n' > download ↵ file download`

- 原用法：`file ~/ops-lab/check.sh` — 按内容识别实验脚本类型。
- 参数变化：`file ~/ops-lab/download` — 识别没有扩展名的实验下载文件。
- 小任务迁移：`file -bi ~/ops-lab/report.txt` — 查看实验文本 MIME 类型和猜测的编码。

## 130. 把三个无空格示例词整理为命令参数

组：`practice-lesson-v03_24_pipeline_control-01`

核心能力：通过管道将输入列表交给 xargs 组织为命令参数。

教学示例：`printf '%s\n' red green blue | xargs echo`

- 原用法：`printf '%s\n' alpha beta | xargs echo` — 将两行实验词汇汇成 echo 参数。
- 参数变化：`printf '%s\n' one two three | xargs -n 1 echo` — 每次只把一个参数交给 echo。
- 小任务迁移：`find ~/ops-lab -type f -print0 | xargs -0 -r ls -l` — 用 NUL 分隔可靠传递含空格文件名，空输入不运行 ls。

## 131. 分页阅读较长的进程列表

组：`practice-lesson-v03_24_pipeline_control-04`

核心能力：通过管道把较长输出交给 less 分页阅读。

教学示例：`ps aux | less`

- 原用法：`ps -ef | less` — 把完整进程表交给 less 分页阅读。
- 参数变化：`ls -la /etc | less` — 分页检查配置目录条目，使用 q 退出。
- 小任务迁移：`journalctl -u nginx --no-pager | less` — 关闭 journalctl 自带分页后，把日志显式交给 less。

## 132. 只查看进程列表开头五行

组：`practice-lesson-v03_24_pipes_tee-01`

核心能力：使用管道和 head -n 截取上游输出的开头若干行。

教学示例：`ps aux | head -n 5`

- 原用法：`ps -ef | head -n 4` — 将进程列表交给 head 只显示前四行。
- 参数变化：`ls -lt /var/log | head -n 8` — 截取修改时间排序后的日志列表前八行。
- 小任务迁移：`sort -n latency_ms.txt | head -n 3` — 从实验耗时排序结果取最小的三项。

## 133. 统计日志中错误记录的行数

组：`practice-lesson-v03_24_pipes_tee-02`

核心能力：先 grep 筛选记录，再用 wc -l 统计匹配行数。

教学示例：`grep 'ERROR' app.log | wc -l`

- 原用法：`grep 'ERROR' backend.log | wc -l` — 计算实验后端 ERROR 匹配行数。
- 参数变化：`grep 'WARN' backend.log | wc -l` — 把匹配的警告行交给 wc 统计。
- 小任务迁移：`grep -i 'timeout' backend.log | wc -l` — 忽略大小写筛选超时记录并统计行数。

## 134. 在唯一私有练习目录中演示覆盖重定向

组：`practice-lesson-v03_24_redirection_streams-01`

核心能力：使用单个 > 覆盖写入标准输出。

教学示例：`umask 077 ↵ practice_dir=$(mktemp -d) ↵ chmod 700 "$practice_dir" ↵ echo 'mode=production' > "$practice_dir/app.conf"`

- 原用法：`printf 'mode=development\n' > ~/ops-lab/app.conf` — 把实验配置写入文件，覆盖已有内容。
- 参数变化：`date -u > ~/ops-lab/checked-at.txt` — 保存 UTC 检查时间到实验文件。
- 小任务迁移：`printf 'port=8000\n' > ~/ops-lab/port.conf` — 使用覆盖重定向写入另一项实验配置。

## 135. 在唯一私有日志夹具中演示追加重定向

组：`practice-lesson-v03_24_redirection_streams-02`

核心能力：使用 >> 把标准输出追加到既有文件。

教学示例：`umask 077 ↵ practice_dir=$(mktemp -d) ↵ chmod 700 "$practice_dir" ↵ printf 'deployment started\n' > "$practice_dir/deploy.log" ↵ date >> "$practice_dir/deploy.log"`

- 原用法：`date -u >> ~/ops-lab/deploy.log` — 在实验日志末尾追加 UTC 时间。
- 参数变化：`printf 'started\n' >> ~/ops-lab/deploy.log` — 向实验部署日志追加启动标记。
- 小任务迁移：`printf 'finished\n' >> ~/ops-lab/deploy.log` — 追加结束标记，保留之前内容。

## 136. 把文件内容作为排序操作的标准输入

组：`practice-lesson-v03_24_redirection_streams-04`

核心能力：使用 < 将文件接到程序标准输入。

教学示例：`sort < names.txt`

- 原用法：`sort < ~/ops-lab/hosts.txt` — 从重定向输入读取实验主机列表并排序。
- 参数变化：`wc -l < ~/ops-lab/notes.txt` — 通过标准输入统计实验笔记行数。
- 小任务迁移：`python3 -m json.tool < ~/ops-lab/response.json` — 通过标准输入验证和格式化实验 JSON。

## 137. 查看上一条命令是否成功及其退出状态

组：`practice-lesson-v03_25_alias_source_shell-01`

核心能力：及时读取 $? 以观察前一命令退出状态。

教学示例：`echo $?`

- 原用法：`printf 'exit=%s\n' "$?"` — 读取最近命令的退出状态，后续命令会更新该状态。
- 参数变化：`true; printf 'exit=%s\n' "$?"` — 观察成功命令的退出状态 0。
- 小任务迁移：`false; printf 'exit=%s\n' "$?"` — 观察 false 的退出状态 1。

## 138. 在交互会话中查看当前 shell 的调用名

组：`practice-lesson-v03_25_alias_source_shell-02`

核心能力：读取 $0，理解当前 shell 或 bash -c 调用名。

教学示例：`echo $0`

- 原用法：`printf '%s\n' "$0"` — 在当前 shell 显示启动名称，值不一定是可执行文件路径。
- 参数变化：`bash -c 'printf "%s\n" "$0"' demo` — 给 bash -c 提供明确的零号参数。
- 小任务迁移：`bash -c 'printf "%s\n" "$0"' health-check` — 用另一实验脚本标识理解 $0。

## 139. 查看 Bash 版本与构建平台

组：`practice-lesson-v03_25_alias_source_shell-03`

核心能力：读取 Bash 本身的版本信息。

教学示例：`bash --version`

- 原用法：`printf '%s\n' "$BASH_VERSION"` — 在 Bash 中读取版本变量。
- 参数变化：`bash --version | head -n 1` — 只读取 PATH 中 Bash 程序的版本首行。
- 小任务迁移：`LC_ALL=C bash --version` — 在 C 区域下查看 Bash 程序完整版本说明。

## 140. 用双引号安全展开并显示用户家目录

组：`practice-lesson-v03_25_environment_variables-01`

核心能力：展开 HOME，查看或构造当前用户主目录路径。

教学示例：`echo "$HOME"`

- 原用法：`printf '%s\n' "$HOME"` — 安全展开当前用户主目录变量。
- 参数变化：`printenv HOME` — 读取导出给子进程的HOME环境变量。
- 小任务迁移：`printf 'config=%s\n' "$HOME/.config"` — 使用HOME构造用户配置目录并显示。

## 141. 用单引号原样显示变量名而不展开

组：`practice-lesson-v03_25_environment_variables-02`

核心能力：用单引号保护变量名，显示字面 $NAME 而不展开。

教学示例：`echo '$HOME'`

- 原用法：`echo '$USER'` — 用单引号阻止用户变量展开。
- 参数变化：`echo '$SHELL'` — 打印变量名文本而不求值。
- 小任务迁移：`printf '%s\n' '$PATH'` — 把 PATH 变量名作为字面字符串输出。

## 142. 显示当前会话的用户名变量

组：`practice-lesson-v03_25_environment_variables-03`

核心能力：展开或读取 USER 用户名环境变量。

教学示例：`echo "$USER"`

- 原用法：`printf '%s\n' "$USER"` — 展开当前会话的用户名变量。
- 参数变化：`printenv USER` — 从导出的环境中读取USER。
- 小任务迁移：`printf 'operator=%s\n' "$USER"` — 将用户名变量加上操作者标签供检查记录。

## 143. 显示用户登录 shell 的完整路径

组：`practice-lesson-v03_25_environment_variables-04`

核心能力：展开或读取 SHELL 登录 shell 路径变量。

教学示例：`echo "$SHELL"`

- 原用法：`printf '%s\n' "$SHELL"` — 读取登录shell路径变量，不据此假定当前解释器。
- 参数变化：`printenv SHELL` — 从导出的环境变量读取登录shell路径。
- 小任务迁移：`printf 'login_shell=%s\n' "$SHELL"` — 带明确标签输出登录shell路径用于环境诊断。

## 144. 只查看已导出的 PATH 环境变量值

组：`practice-lesson-v03_25_path_command_diagnostics-04`

核心能力：读取已导出的 PATH 命令搜索目录列表。

教学示例：`printenv PATH`

- 原用法：`printenv -- PATH` — 查看子进程可继承的 PATH 搜索路径。
- 参数变化：`printenv PATH | tr ':' '\n'` — 将已导出的 PATH 按冒号拆为每行一个搜索目录。
- 小任务迁移：`env | grep '^PATH='` — 从导出的环境变量中精确筛出 PATH 项。

## 145. 按名称查找文档目录中的 PDF 文件

组：`practice-lesson-v03_26_find_names_types-01`

核心能力：使用 find 的名称匹配筛选普通文件。

教学示例：`find ~/Documents -type f -name '*.pdf'`

- 原用法：`find ~/ops-lab -type f -name '*.txt'` — 在实验目录查找 txt 普通文件，引号阻止 shell 提前展开。
- 参数变化：`find ~/ops-lab -type f -iname '*.LOG'` — 忽略大小写查找实验日志扩展名。
- 小任务迁移：`find ~/ops-lab -maxdepth 2 -type f -name '*.pdf'` — 在实验目录两层以内按名称模式查找 PDF 普通文件，引号防止 shell 提前展开。

## 146. 忽略大小写查找下载目录中的 ZIP 包

组：`practice-lesson-v03_26_find_names_types-02`

核心能力：使用 find -iname 忽略文件名大小写。

教学示例：`find ~/Downloads -type f -iname '*.zip'`

- 原用法：`find ~/ops-lab -type f -iname '*.ZIP'` — 忽略扩展名大小写查找实验压缩包。
- 参数变化：`find ~/ops-lab -type f -iname 'readme*'` — 大小写不敏感地匹配实验说明文件前缀。
- 小任务迁移：`find ~/ops-lab/input -type f -iname '*.PDF'` — 在实验输入目录查找各种大小写的 PDF 扩展名。

## 147. 递归列出 src 中的普通文件

组：`practice-lesson-v03_26_find_names_types-03`

核心能力：使用 find -type f 只选择普通文件。

教学示例：`find src -type f`

- 原用法：`find ~/ops-lab/input -type f` — 列出实验输入目录中的普通文件。
- 参数变化：`find ~/ops-lab/backup -type f` — 列出实验备份目录中的普通文件。
- 小任务迁移：`find ~/ops-lab -maxdepth 2 -type f` — 限制深度为两层，只选择普通文件。

## 148. 递归查找配置目录中的 api_url 设置

组：`practice-lesson-v03_26_grep_context_regex-01`

核心能力：使用 grep -r 或 -R 递归搜索目录树。

教学示例：`grep -r 'api_url' config`

- 原用法：`grep -r 'listen' ~/ops-lab/config` — 递归查找实验配置中的监听设置。
- 参数变化：`grep -rn 'timeout' ~/ops-lab/config` — 递归匹配实验超时设置并附带行号。
- 小任务迁移：`grep -r 'database_url' ~/ops-lab/config` — 在配置目录树中查找实验数据库连接设置。

## 149. 显示错误日志及其行号

组：`practice-lesson-v03_26_grep_context_regex-02`

核心能力：使用 grep -n 显示匹配记录的原始行号。

教学示例：`grep -n 'ERROR' app.log`

- 原用法：`grep -n 'ERROR' backend.log` — 定位后端 ERROR 记录和行号。
- 参数变化：`grep -n 'WARN' backend.log` — 显示实验警告记录及行号。
- 小任务迁移：`grep -n -F 'request_id=abc123' backend.log` — 用固定字符串定位实验请求，并保留行号。

## 150. 忽略大小写查找警告日志

组：`practice-lesson-v03_26_grep_context_regex-03`

核心能力：使用 grep -i 进行大小写不敏感匹配。

教学示例：`grep -i 'warning' server.log`

- 原用法：`grep -i 'timeout' backend.log` — 忽略大小写筛选实验超时记录。
- 参数变化：`grep -in 'warning' backend.log` — 忽略大小写匹配警告并显示行号。
- 小任务迁移：`grep -i 'connection refused' backend.log` — 大小写不敏感地筛选实验连接被拒绝日志。

## 151. 已安装且索引可用时，按路径名快速定位 SSH 配置

组：`practice-lesson-v03_26_locate_command_discovery-01`

核心能力：用 locate 从已有路径索引中查找名称。

教学示例：`locate ssh_config`

- 原用法：`locate sshd_config` — 从文件索引查找 SSH 服务端配置，结果可能滞后于磁盘。
- 参数变化：`locate -b '\nginx.conf'` — 按基本文件名查找 nginx 配置。
- 小任务迁移：`locate -e hosts` — 只显示索引中当前仍存在的匹配路径。

## 152. 只读检查系统是否提供 locate 索引更新程序

组：`practice-lesson-v03_26_locate_command_discovery-02`

核心能力：从 PATH 定位指定外部工具并确认是否安装。

教学示例：`command -v updatedb`

- 原用法：`command -v python3` — 从当前PATH定位Python外部程序。
- 参数变化：`which tar` — 使用which定位归档工具的外部路径。
- 小任务迁移：`command -v curl` — 在运行网络检查前确认curl能否从PATH找到。

## 153. 查看磁盘层级、容量和挂载点

组：`practice-lesson-v03_27_block_mount_identity-01`

核心能力：用 lsblk 同时读取设备名称、类型、大小和挂载位置。

教学示例：`lsblk -o NAME,SIZE,TYPE,MOUNTPOINTS`

- 原用法：`lsblk -o NAME,TYPE,SIZE,MOUNTPOINTS` — 同时列出块设备层级、类型、容量和挂载位置。
- 参数变化：`lsblk -o NAME,SIZE,TYPE,MOUNTPOINTS /dev/sda` — 限定实验磁盘并保留层级和挂载字段。
- 小任务迁移：`lsblk --json -o NAME,SIZE,TYPE,MOUNTPOINTS` — 用JSON表示同样的设备层级和挂载信息。

## 154. 查看块设备的文件系统标识

组：`practice-lesson-v03_27_block_mount_identity-02`

核心能力：用 lsblk 的文件系统字段识别类型和挂载关系。

教学示例：`lsblk -f`

- 原用法：`lsblk -f /dev/sda` — 读取实验磁盘的文件系统类型、标识和挂载信息。
- 参数变化：`lsblk -o NAME,FSTYPE,UUID,MOUNTPOINTS` — 显式选择文件系统识别所需字段。
- 小任务迁移：`lsblk -f -e 7` — 忽略loop设备主设备号7后查看文件系统标识。

## 155. 精确查看根目录的挂载信息

组：`practice-lesson-v03_27_block_mount_identity-04`

核心能力：用 findmnt 将路径定位到实际挂载点。

教学示例：`findmnt /`

- 原用法：`findmnt -T /var/log` — 查明路径所在的文件系统。
- 参数变化：`findmnt -T /srv/app` — 定位实验部署目录使用的文件系统。
- 小任务迁移：`findmnt -o TARGET,SOURCE,FSTYPE /` — 查看根挂载的来源与文件系统类型。

## 156. 查看根文件系统的容量和可用空间

组：`practice-lesson-v03_27_disk_capacity_inodes-01`

核心能力：用 df 读取文件系统字节容量与可用量。

教学示例：`df -h /`

- 原用法：`df -h /var/log` — 以易读单位查看日志路径所在文件系统的字节容量。
- 参数变化：`df -hT /srv/app` — 同时查看应用文件系统类型和字节容量。
- 小任务迁移：`df -h /home` — 核对家目录所在文件系统的容量与可用空间。

## 157. 确认根文件系统类型和块使用量

组：`practice-lesson-v03_27_disk_capacity_inodes-02`

核心能力：用 df -T 同时读取文件系统类型及块容量。

教学示例：`df -T /`

- 原用法：`df -T /var/log` — 显示日志路径所在文件系统类型与块容量。
- 参数变化：`df -hT /srv/app` — 保留类型字段并使用易读容量单位。
- 小任务迁移：`df -T /home` — 核对家目录所在文件系统的类型和使用量。

## 158. 汇总下载目录实际占用空间

组：`practice-lesson-v03_27_du_usage_ranking-01`

核心能力：用 du -s 汇总目录实际磁盘占用并以易读单位显示。

教学示例：`du -sh ~/Downloads`

- 原用法：`du -sh ~/ops-lab/input` — 汇总输入目录实际磁盘占用。
- 参数变化：`du -sh ~/ops-lab/backup` — 改变目标并汇总备份目录占用。
- 小任务迁移：`du -sh -- ~/ops-lab/logs` — 在明确结束选项后汇总日志目录占用。

## 159. 汇总用户缓存目录的实际磁盘占用

组：`practice-lesson-v03_27_du_usage_ranking-04`

核心能力：用 du -s 汇总目录实际磁盘占用并以易读单位显示。

教学示例：`du -sh ~/.cache`

- 原用法：`du -sh ~/ops-lab/input` — 汇总输入目录实际磁盘占用。
- 参数变化：`du -sh ~/ops-lab/backup` — 改变目标并汇总备份目录占用。
- 小任务迁移：`du -sh -- ~/ops-lab/logs` — 在明确结束选项后汇总日志目录占用。

## 160. 汇总视频目录的实际磁盘占用

组：`practice-lesson-v03_27_du_usage_ranking-05`

核心能力：用 du -s 汇总目录实际磁盘占用并以易读单位显示。

教学示例：`du -sh ~/Videos`

- 原用法：`du -sh ~/ops-lab/input` — 汇总输入目录实际磁盘占用。
- 参数变化：`du -sh ~/ops-lab/backup` — 改变目标并汇总备份目录占用。
- 小任务迁移：`du -sh -- ~/ops-lab/logs` — 在明确结束选项后汇总日志目录占用。

## 161. 只列出当前 shell 各活动作业的进程号

组：`practice-lesson-v03_28_ports_signals_jobs-05`

核心能力：使用 jobs -p 只列出当前 shell 作业的 PID。

教学示例：`jobs -p`

- 原用法：`jobs -p %1` — 只输出当前 shell 一号作业的进程组首进程 PID。
- 参数变化：`jobs -p %2` — 只读取当前 shell 二号实验作业 PID。
- 小任务迁移：`jobs -rp` — 仅输出当前 shell 正在运行作业的 PID。

## 162. 让编号 2 的已暂停作业在后台继续运行

组：`practice-lesson-v03_28_ports_signals_jobs-06`

核心能力：使用 bg 将已暂停的当前 shell 作业恢复到后台。

教学示例：`bg %2`

- 原用法：`bg %1` — 将当前shell已暂停的作业1恢复到后台。
- 参数变化：`bg %3` — 改变作业号以恢复课程预置的作业3。
- 小任务迁移：`bg %4` — 把课程预置的暂停作业4恢复到后台继续处理。

## 163. 把编号 2 的后台作业调回前台

组：`practice-lesson-v03_28_ports_signals_jobs-07`

核心能力：使用 fg 将当前 shell 的作业调到前台。

教学示例：`fg %2`

- 原用法：`fg %1` — 把当前shell的作业1恢复到前台。
- 参数变化：`fg %3` — 改变作业号并接回作业3。
- 小任务迁移：`fg %4` — 回到课程预置作业4的前台交互。

## 164. 以易读格式显示系统运行时长

组：`practice-lesson-v03_28_process_inspection-01`

核心能力：使用 uptime 的 pretty 模式读取易读的连续运行时长。

教学示例：`uptime -p`

- 原用法：`uptime --pretty` — 使用长选项输出易读的连续运行时长。
- 参数变化：`LC_ALL=C uptime -p` — 固定输出语言后读取pretty运行时长。
- 小任务迁移：`uptime -p | sed 's/^up //'` — 读取易读运行时长并去掉开头标签供显示。

## 165. 查看全量进程并按 CPU 使用率降序排列

组：`practice-lesson-v03_28_process_inspection-02`

核心能力：读取全部进程并按 CPU 使用率降序排列。

教学示例：`ps aux --sort=-%cpu`

- 原用法：`ps -eo pid,user,%cpu,args --sort=-%cpu` — 读取全部进程并按CPU占用降序排列。
- 参数变化：`ps -eo pid,%cpu,%mem,args --sort=-%cpu` — 保留全量CPU降序并增加内存字段。
- 小任务迁移：`ps -eo pid,comm,%cpu --sort=-%cpu | head -n 6` — 从全量CPU排序中截取表头和前五个进程。

## 166. 只读查看 chen 名下进程的 PID、状态和命令行

组：`practice-lesson-v03_28_process_inspection-03`

核心能力：按明确用户名筛选进程并读取 PID、状态与命令行。

教学示例：`ps -u chen -o pid,stat,cmd`

- 原用法：`ps -u www-data -o pid,stat,args` — 筛选Web用户进程并显示PID、状态与命令行。
- 参数变化：`ps -u ace -o pid,stat,args` — 改变用户名以查看课程账号的同一组字段。
- 小任务迁移：`ps -u root -o pid,stat,args --sort=pid` — 只读核对root进程并按PID排列其状态与命令行。

## 167. 从网址下载源码归档到当前目录

组：`practice-lesson-v03_29_network_downloads-01`

核心能力：通过 wget 下载响应正文到本地文件。

教学示例：`wget https://ftp.gnu.org/gnu/hello/hello-2.12.2.tar.gz`

- 原用法：`wget -O example.html https://example.com/` — 下载示例网页并指定本地输出文件名。
- 参数变化：`wget --timeout=5 --tries=1 https://example.com/` — 限制下载等待与重试次数，避免持续等待。
- 小任务迁移：`wget -O hello.tar.gz https://ftp.gnu.org/gnu/hello/hello-2.12.2.tar.gz` — 从 GNU 官方站点下载实验归档并指定本地名称。

## 168. 只查看网站的 HTTP 响应头

组：`practice-lesson-v03_29_network_downloads-04`

核心能力：使用 curl -I 发送 HEAD 请求只读取响应头。

教学示例：`curl -I https://www.kernel.org`

- 原用法：`curl -I https://example.com/` — 只获取 IANA 保留示例域名根页面的 HTTP 响应头。
- 参数变化：`curl -I --max-time 5 https://example.com/` — 用实际 HTTPS 请求交叉验证，避免只依赖 ping。
- 小任务迁移：`curl -I https://www.kernel.org/` — 对另一官方站点只读取 HTTP 响应头。

## 169. 发送四次请求检查网络连通与时延

组：`practice-lesson-v03_29_network_downloads-05`

核心能力：用有限次数的 ICMP 回显采样连通性与往返时间。

教学示例：`ping -c 4 1.1.1.1`

- 原用法：`ping -c 2 192.0.2.1` — 向实验网关发两次 ICMP 回显请求。
- 参数变化：`ping -c 5 -W 1 192.0.2.1` — 采样五次并限制每次等待，比较丢包情况。
- 小任务迁移：`ping -n -c 3 198.51.100.20` — 直接测试实验 IP 并禁用名称解析，隔离 DNS 干扰。

## 170. 查看所有网络接口及其地址

组：`practice-lesson-v03_29_network_downloads-06`

核心能力：通过 ip address 查询接口的 IPv4/IPv6 地址。

教学示例：`ip a`

- 原用法：`ip -br address show dev ens33` — 以简表查看实验网卡地址。
- 参数变化：`ip -4 address show dev ens33` — 仅检查实验网卡 IPv4 地址。
- 小任务迁移：`ip -6 address show dev ens33` — 检查实验网卡 IPv6 地址及 tentative、dadfailed 等标志。

## 171. 确认远端无同名文件后上传本地报告

组：`practice-lesson-v03_29_scp_rsync-01`

核心能力：用 scp 从本地上传到明确远端路径。

教学示例：`scp report.pdf alice@server.example.com:/home/alice/`

- 原用法：`scp ~/ops-lab/report.txt ops@192.0.2.20:/tmp/report.txt` — 上传实验报告到明确远端路径。
- 参数变化：`scp -P 2222 ~/ops-lab/settings.ini ops@192.0.2.20:/tmp/settings.ini` — 使用大写 -P 指定实验 SSH 端口。
- 小任务迁移：`scp ~/ops-lab/notes.txt alice@server.example.com:/home/alice/notes.txt` — 以域名指定服务器并上传本地实验笔记。

## 172. 确认本地无同名文件后下载远程报表

组：`practice-lesson-v03_29_scp_rsync-02`

核心能力：用 scp 从明确远端路径下载到本地。

教学示例：`scp alice@server.example.com:/home/alice/report.csv .`

- 原用法：`scp ops@192.0.2.20:/tmp/report.txt ~/ops-lab/input/` — 把远端实验报告下载到本地输入目录。
- 参数变化：`scp -P 2222 ops@192.0.2.20:/tmp/settings.ini ~/ops-lab/input/` — 通过指定 SSH 端口下载实验配置。
- 小任务迁移：`scp alice@server.example.com:/home/alice/notes.txt ~/ops-lab/input/` — 从域名指定的服务器下载实验笔记。

## 173. 首次登录先通过可信渠道核验主机指纹

组：`practice-lesson-v03_29_ssh_hosts_keys-01`

核心能力：首次 SSH 登录保留主机密钥校验，并通过独立可信渠道比对指纹。

教学示例：`ssh alice@server.example.com`

- 原用法：`ssh ops@192.0.2.20` — 以 ops 账号连接实验服务器。
- 参数变化：`ssh -p 2222 ops@192.0.2.20` — 使用实验 SSH 非默认端口连接。
- 小任务迁移：`ssh -o ConnectTimeout=5 ops@192.0.2.20` — 限制实验 SSH 连接等待为五秒。

## 174. 分页查看个人定时任务并保留错误诊断

组：`practice-lesson-v03_30_cron_scheduling-02`

核心能力：读取个人 crontab 并用 less 分页观察。

教学示例：`crontab -l | less`

- 原用法：`crontab -l | less -N` — 读取个人定时配置并带行号分页。
- 参数变化：`crontab -l | less +G` — 分页打开个人定时配置后先定位末尾。
- 小任务迁移：`crontab -l | less -S` — 分页检查较长的定时命令并避免自动折行。

## 175. 仅打印每小时第 15 分钟运行的示例

组：`practice-lesson-v03_30_cron_scheduling-04`

核心能力：只打印含五个时间字段和命令的 cron 配置示例。

教学示例：`echo '15 * * * * /usr/local/bin/health-check'`

- 原用法：`printf '%s\n' '30 * * * * /usr/local/bin/health-check'` — 只打印每小时第 30 分执行的实验条目，不安装任务。
- 参数变化：`printf '%s\n' '0 2 * * * /usr/local/bin/backup'` — 只打印每天 02:00 的实验备份条目。
- 小任务迁移：`printf '%s\n' '*/5 * * * * /usr/local/bin/health-check'` — 只打印每五分钟检查的条目，理解分钟字段步长。

## 176. 只读查看 cron 守护进程状态与近期日志

组：`practice-lesson-v03_30_cron_scheduling-05`

核心能力：用 systemctl status 阅读指定服务状态与最近诊断。

教学示例：`systemctl status cron.service`

- 原用法：`systemctl status nginx.service --no-pager` — 查看实验 Web 服务当前状态。
- 参数变化：`systemctl status ssh.service --no-pager` — 查看 Ubuntu SSH 服务状态。
- 小任务迁移：`systemctl status cron.service --no-pager --full` — 完整查看 Ubuntu 定时任务服务状态。

## 177. 只输出 cron 服务当前是否处于活动状态

组：`practice-lesson-v03_30_cron_scheduling-06`

核心能力：只查询服务当前是否处于 active 状态。

教学示例：`systemctl is-active cron.service`

- 原用法：`systemctl is-active ssh.service` — 只读取SSH服务当前活动状态。
- 参数变化：`systemctl is-active nginx.service` — 以相同操作检查Nginx当前状态。
- 小任务迁移：`systemctl is-active rsyslog.service` — 在日志诊断前查询rsyslog是否活动。

## 178. 只输出 cron 服务是否设为开机启用

组：`practice-lesson-v03_30_cron_scheduling-07`

核心能力：只查询服务的开机启用状态。

教学示例：`systemctl is-enabled cron.service`

- 原用法：`systemctl is-enabled nginx.service` — 读取Nginx服务的开机启用状态。
- 参数变化：`systemctl is-enabled ssh.service` — 查询SSH服务是否配置为开机启用。
- 小任务迁移：`systemctl is-enabled systemd-timesyncd.service` — 核对时间同步服务的启用配置。

## 179. 查看当前账号有权读取的 nginx 服务日志

组：`practice-lesson-v03_30_journal_logs-01`

核心能力：按 systemd 单元筛选并读取该服务的 journal 日志。

教学示例：`journalctl -u nginx.service`

- 原用法：`journalctl -u ssh.service -n 10 --no-pager` — 查看实验 SSH 最近十条服务日志。
- 参数变化：`journalctl -u nginx.service -n 20 --no-pager` — 查看实验 Web 最近二十条服务日志。
- 小任务迁移：`journalctl -u cron.service --since '-1 hour' --no-pager` — 限定到最近一小时的定时任务服务日志。

## 180. 查看 SSH 服务的运行状态与近期日志

组：`practice-lesson-v03_30_systemd_service_inspection-01`

核心能力：用 systemctl status 阅读指定服务状态与最近诊断。

教学示例：`systemctl status ssh.service`

- 原用法：`systemctl status nginx.service --no-pager` — 查看实验 Web 服务当前状态。
- 参数变化：`systemctl status ssh.service --no-pager` — 查看 Ubuntu SSH 服务状态。
- 小任务迁移：`systemctl status cron.service --no-pager --full` — 完整查看 Ubuntu 定时任务服务状态。

## 181. 筛出当前处于失败状态的服务

组：`practice-lesson-v03_30_systemd_service_inspection-08`

核心能力：筛选 systemd 的失败单元列表。

教学示例：`systemctl list-units --type=service --state=failed`

- 原用法：`systemctl --failed --no-pager` — 只列出当前失败单元并关闭分页。
- 参数变化：`systemctl list-units --type=service --state=failed --no-pager` — 将失败列表限定为服务单元。
- 小任务迁移：`systemctl list-units --failed --type=service --no-legend` — 去掉说明行后查看失败服务清单。

## 182. 只读确认 nano 是否安装及其版本

组：`practice-lesson-v03_31_nano_survival-01`

核心能力：读取 nano 版本信息而不进入编辑。

教学示例：`nano --version`

- 原用法：`nano --version | head -n 1` — 只读取 nano 版本号所在首行。
- 参数变化：`nano --version | head -n 3` — 查看 nano 版本输出开头的构建说明。
- 小任务迁移：`LC_ALL=C nano --version` — 在 C 区域下查看 nano 完整版本信息。

## 183. 用带底部快捷键提示的编辑器打开待办文件

组：`practice-lesson-v03_31_nano_survival-02`

核心能力：使用 nano 打开可编辑的练习文本。

教学示例：`nano todo.txt`

- 原用法：`nano ~/ops-lab/notes.txt` — 打开可编辑的练习笔记并查看底部快捷键提示。
- 参数变化：`nano -l ~/ops-lab/check.sh` — 编辑脚本时增加行号帮助定位。
- 小任务迁移：`nano -w ~/ops-lab/settings.ini` — 编辑练习配置并关闭自动硬换行。

## 184. 编辑脚本时在文本区左侧显示行号

组：`practice-lesson-v03_31_nano_survival-05`

核心能力：使用 nano -l 在编辑时显示行号。

教学示例：`nano -l deploy.sh`

- 原用法：`nano -l ~/ops-lab/check.sh` — 保留 nano -l 的行号显示，在练习脚本中按行定位编辑。
- 参数变化：`nano -l ~/ops-lab/settings.ini` — 带行号查看和编辑实验配置。
- 小任务迁移：`nano -l ~/ops-lab/notes.txt` — 使用行号在实验笔记中定位内容。

## 185. 用 Vim 打开笔记；进入后按 i 编辑，Esc 后用 :wq 保存退出

组：`practice-lesson-v03_31_vim_navigation_editing-01`

核心能力：使用 Vim 打开可编辑的练习文本。

教学示例：`vim notes.txt`

- 原用法：`vim ~/ops-lab/notes.txt` — 打开练习笔记后可按i编辑并用:wq保存退出。
- 参数变化：`vim +12 ~/ops-lab/check.sh` — 打开可编辑脚本并先定位第十二行。
- 小任务迁移：`vim -n ~/ops-lab/settings.ini` — 在临时练习配置中编辑且不创建交换文件。

## 186. 只读分页查看脚本并显示行号

组：`practice-lesson-v03_31_vim_navigation_editing-02`

核心能力：使用 less -N 分页显示文本并保留行号。

教学示例：`less -N deploy.sh`

- 原用法：`less -N ~/ops-lab/check.sh` — 以带行号模式只读分页浏览脚本。
- 参数变化：`less -N ~/ops-lab/backend.log` — 使用 less -N 分页检查后端日志，并保留原始行号。
- 小任务迁移：`less -N +G ~/ops-lab/backend.log` — 带行号打开日志并先定位末尾。

## 187. 只读分页查看源码，进入后可用 /TODO 搜索

组：`practice-lesson-v03_31_vim_navigation_editing-03`

核心能力：使用 less 只读分页浏览文本，可定位或搜索。

教学示例：`less src/main.rs`

- 原用法：`less -N ~/ops-lab/check.sh` — 带行号阅读实验脚本，q 退出。
- 参数变化：`less ~/ops-lab/backend.log` — 分页阅读实验后端日志。
- 小任务迁移：`less +G ~/ops-lab/backend.log` — 打开实验日志并先跳到末尾。

## 188. 只读分页查看较长的网络服务名称表

组：`practice-lesson-v03_31_vim_navigation_editing-04`

核心能力：使用 less 只读分页浏览文本，可定位或搜索。

教学示例：`less /etc/services`

- 原用法：`less -N ~/ops-lab/check.sh` — 带行号阅读实验脚本，q 退出。
- 参数变化：`less ~/ops-lab/backend.log` — 分页阅读实验后端日志。
- 小任务迁移：`less +G ~/ops-lab/backend.log` — 打开实验日志并先跳到末尾。

## 189. 修改前先只读分页检查 nginx 配置

组：`practice-lesson-v03_31_vim_navigation_editing-05`

核心能力：使用 less 只读分页浏览文本，可定位或搜索。

教学示例：`less nginx.conf`

- 原用法：`less -N ~/ops-lab/check.sh` — 带行号阅读实验脚本，q 退出。
- 参数变化：`less ~/ops-lab/backend.log` — 分页阅读实验后端日志。
- 小任务迁移：`less +G ~/ops-lab/backend.log` — 打开实验日志并先跳到末尾。

## 190. 不进编辑器，直接查看脚本并给所有行编号

组：`practice-lesson-v03_31_vim_navigation_editing-07`

核心能力：使用 cat -n 给全部输出行编号。

教学示例：`cat -n deploy.sh`

- 原用法：`cat -n ~/ops-lab/settings.ini` — 带行号读取实验配置，方便交流具体位置。
- 参数变化：`cat -n ~/ops-lab/check.sh` — 为实验脚本每一行编号。
- 小任务迁移：`cat -n ~/ops-lab/notes.txt` — 给实验笔记加行号，方便交流位置。

## 191. 用 Vim 的只读入口查看 hosts，减少误写风险

组：`practice-lesson-v03_31_vim_recovery_readonly-01`

核心能力：通过 view 或 vim -R 以只读模式打开文件。

教学示例：`view /etc/hosts`

- 原用法：`view ~/ops-lab/settings.ini` — 以只读模式查看实验配置，仍需了解强制写入可绕过只读标志。
- 参数变化：`vim -R ~/ops-lab/check.sh` — 显式使用只读标志查看实验脚本。
- 小任务迁移：`view +12 ~/ops-lab/backend.log` — 只读打开实验日志并先定位第十二行。

## 192. 只读确认 Vim 版本和已编译功能

组：`practice-lesson-v03_31_vim_recovery_readonly-02`

核心能力：读取 Vim 版本和编译能力信息。

教学示例：`vim --version`

- 原用法：`vim --version | head -n 1` — 读取 Vim 版本输出的首行。
- 参数变化：`vim --version | grep -E 'python|clipboard'` — 从版本功能表检查 Python 与剪贴板支持标记。
- 小任务迁移：`LC_ALL=C vim --version` — 在 C 区域下读取完整 Vim 版本及编译信息。

## 193. 只读确认通用 editor 命令最终指向哪个程序

组：`practice-lesson-v03_31_vim_recovery_readonly-03`

核心能力：用 readlink -f 解析编辑器入口的符号链接链。

教学示例：`readlink -f /usr/bin/editor`

- 原用法：`readlink -f /usr/bin/vi` — 解析通用 vi 入口的链接链并显示实际编辑器路径。
- 参数变化：`readlink -f /usr/bin/ex` — 确认 ex 兼容编辑器入口最终指向哪个程序。
- 小任务迁移：`readlink -f "$(command -v editor)"` — 先定位 editor 入口，再解析为真正的编辑器程序。

## 194. 卡在启动选项时查看 Vim 简短帮助

组：`practice-lesson-v03_31_vim_recovery_readonly-04`

核心能力：读取 Vim 的命令行选项帮助。

教学示例：`vim -h`

- 原用法：`vim --help` — 阅读 Vim 命令行选项帮助。
- 参数变化：`vim --help | head -n 12` — 只查看帮助开始十二行，先掌握基本启动方式。
- 小任务迁移：`vim --help | less` — 分页阅读 Vim 帮助，使用 q 退出分页器。

## 195. 让启动脚本仅所有者可写，所有用户可执行

组：`practice-lesson-v03_32_permissions_ownership-01`

核心能力：设置所有者 rwx、组和其他人 rx 的 755 权限。

教学示例：`chmod 755 training_lab/bin/start.sh`

- 原用法：`chmod 755 ~/ops-lab/check.sh` — 设置练习脚本为所有者 rwx、组与其他人 rx。
- 参数变化：`chmod -v 755 ~/ops-lab/deploy.sh` — 以详细输出确认另一脚本的 755 执行权限。
- 小任务迁移：`chmod u=rwx,go=rx ~/ops-lab/start.sh` — 用等价符号权限表达 755，使启动脚本仅所有者可写。

## 196. 让笔记仅所有者可写，所有用户可读

组：`practice-lesson-v03_32_permissions_ownership-02`

核心能力：设置所有者可读写、组及其他人只读的 644 权限。

教学示例：`chmod 644 training_lab/notes.txt`

- 原用法：`chmod 644 ~/ops-lab/notes.txt` — 将练习笔记设为所有者读写、组与其他人只读。
- 参数变化：`chmod -v 644 ~/ops-lab/report.txt` — 设置报告为 644 并显示权限变更。
- 小任务迁移：`chmod u=rw,go=r ~/ops-lab/public.conf` — 用符号形式设置不含秘密的公开练习配置为 644。

## 197. 在私有夹具中只读核对文件所有者和组

组：`practice-lesson-v03_32_sudo_audit-01`

核心能力：只读检查文件或目录的所有者和所属组。

教学示例：`umask 077 ↵ practice_dir=$(mktemp -d) ↵ chmod 700 "$practice_dir" ↵ cd "$practice_dir" ↵ mkdir ownership ↵ printf 'notes\n' > ownership/notes.txt ↵ stat -c '%U:%G %n' ownership/notes.txt`

- 原用法：`stat -c '%U:%G %n' ~/ops-lab/report.txt` — 读取实验报告的用户与组所有者。
- 参数变化：`stat -c '%U:%G %a' ~/ops-lab/settings.ini` — 检查实验配置所有者、组与数字权限。
- 小任务迁移：`ls -ld ~/ops-lab/backup` — 查看实验备份目录本身的所有者与组。

## 198. 只读确认 training 账号是否存在及其登录信息

组：`practice-lesson-v03_32_users_identity_groups-01`

核心能力：从 NSS passwd 数据库查询用户的 UID、主组和登录字段。

教学示例：`getent passwd training`

- 原用法：`getent passwd ace` — 通过NSS查询课程账号的登录记录。
- 参数变化：`getent passwd www-data` — 查询Web服务账号的UID、主组和登录shell。
- 小任务迁移：`getent passwd root` — 只读核对管理员账号的passwd身份字段。

## 199. 查看教学账号所属的全部用户组

组：`practice-lesson-v03_32_users_identity_groups-03`

核心能力：读取指定用户所属的全部组。

教学示例：`groups training`

- 原用法：`groups` — 显示当前用户所属的所有用户组。
- 参数变化：`groups www-data` — 读取实验 Web 服务账号所属组。
- 小任务迁移：`id -Gn ace` — 只输出实验 ace 用户所属组名。

## 200. 查看教学账号的 UID、GID 和组成员资格

组：`practice-lesson-v03_32_users_identity_groups-04`

核心能力：通过 id 查询用户身份或其中指定身份字段。

教学示例：`id training`

- 原用法：`id` — 显示当前用户的 UID、GID 和用户组信息。
- 参数变化：`id www-data` — 查看 Web 服务账号身份。
- 小任务迁移：`id -u ace` — 只读取实验 ace 用户的 UID。

## 201. 统计每一段连续重复状态出现了几次

组：`practice-lesson-v03_33_counts_head_tail-01`

核心能力：使用 uniq -c 为连续重复行计数。

教学示例：`uniq -c status_runs.txt`

- 原用法：`uniq -c sorted-hosts.txt` — 为已相邻排列的重复主机名计数。
- 参数变化：`uniq -c status-runs.txt` — 对状态序列的每一段连续重复值计数。
- 小任务迁移：`sort hosts.txt | uniq -c` — 先排序使同名相邻，再统计每个主机名出现次数。

## 202. 只读查看应用日志开头 15 行

组：`practice-lesson-v03_33_counts_head_tail-03`

核心能力：用 head -n 读取文件开头指定行数。

教学示例：`head -n 15 app.log`

- 原用法：`head -n 3 /etc/passwd` — 用 head -n 限定账户文件前三行，保留只读截取开头的操作。
- 参数变化：`head -n 8 ~/ops-lab/input/report.txt` — 把开头行数改为八行，用于检查报告格式和首段内容。
- 小任务迁移：`head -n 1 ~/ops-lab/input/report.csv` — 先查看实验 CSV 表头再分析字段。

## 203. 只读查看服务日志末尾 20 行

组：`practice-lesson-v03_33_counts_head_tail-04`

核心能力：用 tail 读取文件末尾有限行数并退出。

教学示例：`tail -n 20 service.log`

- 原用法：`tail -n 3 backend.log` — 只读取后端日志末尾三行并退出。
- 参数变化：`tail -n 12 backend.log` — 改变行数参数，读取末尾十二行。
- 小任务迁移：`tail -n 5 ~/ops-lab/settings.ini` — 读取配置末尾五行检查最近一段设置。

## 204. 只读统计配置文件的行数

组：`practice-lesson-v03_33_counts_head_tail-05`

核心能力：使用 wc -l 统计文本行数。

教学示例：`wc -l settings.ini`

- 原用法：`wc -l ~/ops-lab/report.txt` — 使用 wc -l 按换行符统计报告行数，不改为字节或词数。
- 参数变化：`wc -l ~/ops-lab/settings.ini` — 保留 wc -l 行数统计，改变目标以核对配置文件规模。
- 小任务迁移：`wc -l ~/ops-lab/notes.txt ~/ops-lab/report.txt` — 分别统计两份实验文件行数和合计。

## 205. 读取组数据库中冒号分隔的组名列

组：`practice-lesson-v03_33_fields_sort_unique-01`

核心能力：使用 cut -d 与 -f 提取分隔文本的指定列。

教学示例：`cut -d: -f1 /etc/group`

- 原用法：`cut -d: -f1 /etc/passwd` — 从 /etc/passwd 中按冒号切出第 1 列。
- 参数变化：`cut -d: -f1,3 /etc/group` — 同时提取组名与 GID。
- 小任务迁移：`cut -d, -f2 ~/ops-lab/report.csv` — 从无嵌套引号逗号的简单 CSV 提取第二列。

## 206. 按数值而非字符顺序排列响应时间

组：`practice-lesson-v03_33_fields_sort_unique-02`

核心能力：使用 sort -n 按数值排序，方向可由 -r 明确改变。

教学示例：`sort -n response_times.txt`

- 原用法：`sort -n latency_ms.txt` — 按数字排序实验耗时值，避免字典序把 100 排在 20 前。
- 参数变化：`sort -nr latency_ms.txt` — 将同一实验数值列表从大到小排序。
- 小任务迁移：`sort -n retry_counts.txt` — 按数值比较实验重试次数，避免字典序误判。

## 207. 按当前语言环境的规则降序排列文本

组：`practice-lesson-v03_33_fields_sort_unique-03`

核心能力：使用 sort -r 对文本进行反向排序。

教学示例：`sort -r regions.txt`

- 原用法：`sort -r hosts.txt` — 按反向字典序排列实验主机名称。
- 参数变化：`sort -r users.txt` — 反向排列实验账号名称。
- 小任务迁移：`sort -ru regions.txt` — 反向排序实验区域名称并合并重复项。

## 208. 把输入中的小写字母转换为大写

组：`practice-lesson-v03_33_join_transform-01`

核心能力：用 tr 将小写字符类转换为大写字符类。

教学示例：`tr '[:lower:]' '[:upper:]' < greeting.txt`

- 原用法：`tr '[:lower:]' '[:upper:]' < ~/ops-lab/notes.txt` — 把练习笔记中的小写字符转换为大写。
- 参数变化：`tr '[:lower:]' '[:upper:]' < ~/ops-lab/labels.txt` — 改变输入文件并保留小写到大写的方向。
- 小任务迁移：`printf 'ready\n' | tr '[:lower:]' '[:upper:]'` — 将管道中的状态词规范为大写。

## 209. 查看应用日志末尾默认 10 行

组：`practice-lesson-v03_33_join_transform-03`

核心能力：用 tail 读取文件末尾有限行数并退出。

教学示例：`tail app.log`

- 原用法：`tail -n 3 backend.log` — 只读取后端日志末尾三行并退出。
- 参数变化：`tail -n 12 backend.log` — 改变行数参数，读取末尾十二行。
- 小任务迁移：`tail -n 5 ~/ops-lab/settings.ini` — 读取配置末尾五行检查最近一段设置。

## 210. 显示当前本地日期、时间和时区缩写

组：`practice-lesson-v03_34_date_apt_sources_locale-01`

核心能力：读取日期时间并展示需要的时区或格式。

教学示例：`date`

- 原用法：`date -u` — 只读显示当前 UTC 日期和时间。
- 参数变化：`date '+%Y-%m-%d %H:%M:%S %z'` — 输出带数字时区的本地完整时间。
- 小任务迁移：`date '+%F %Z'` — 输出日期与时区缩写，跨地区交流优先使用数字偏移。

## 211. 以 YYYY-MM-DD 形式显示今天日期

组：`practice-lesson-v03_34_date_apt_sources_locale-02`

核心能力：使用 date 明确输出 YYYY-MM-DD 日历日期。

教学示例：`date +%F`

- 原用法：`date -u +%F` — 以YYYY-MM-DD输出UTC日历日期。
- 参数变化：`date '+%Y-%m-%d'` — 用等价显式字段输出本地日历日期。
- 小任务迁移：`date -d tomorrow +%F` — 按相同日期格式预览明天的日期。

## 212. 只读显示当前系统时区缩写

组：`practice-lesson-v03_34_date_apt_sources_locale-03`

核心能力：使用 date 的 %Z 读取时区缩写。

教学示例：`date +%Z`

- 原用法：`date -u +%Z` — 保持 %Z 时区缩写字段，只把观察时区改为 UTC。
- 参数变化：`TZ=Asia/Shanghai date +%Z` — 临时按上海时区显示缩写，不修改系统时区。
- 小任务迁移：`TZ=UTC date '+%Z %z'` — 同时显示UTC缩写和偏移以区分两者。

## 213. 在终端显示当前月份的日历

组：`practice-lesson-v03_34_date_apt_sources_locale-05`

核心能力：用 cal 查看月份或年度日历。

教学示例：`cal`

- 原用法：`cal 9 2026` — 查看实验指定月份日历。
- 参数变化：`cal -3` — 显示上月、本月与下月日历。
- 小任务迁移：`cal 2026` — 显示实验指定整年日历。

## 214. 查看语言环境（locale，也称本地化设置）的各分类

组：`practice-lesson-v03_34_locale_variables_encoding-01`

核心能力：用 locale -k 读取当前语言环境的具体分类设置。

教学示例：`locale`

- 原用法：`locale -k LC_TIME` — 读取当前日期时间分类的语言环境设置。
- 参数变化：`locale -k LC_CTYPE` — 读取字符分类与编码相关的当前设置。
- 小任务迁移：`locale -k LC_NUMERIC` — 检查数字小数点和分组规则的当前分类设置。

## 215. 查看 LANG 提供的默认语言环境值

组：`practice-lesson-v03_34_locale_variables_encoding-02`

核心能力：读取 LANG 默认语言环境变量，必要时与覆盖变量比较。

教学示例：`echo "$LANG"`

- 原用法：`printenv LANG` — 读取导出的默认区域设置变量。
- 参数变化：`printf 'LANG=%s\n' "$LANG"` — 带标签输出当前默认区域变量。
- 小任务迁移：`printenv LC_ALL LC_CTYPE LANG` — 同时读取影响字符区域的环境变量，优先级为 LC_ALL、分类变量、LANG。

## 216. 列出本机已经生成并可用的语言环境

组：`practice-lesson-v03_34_locale_variables_encoding-03`

核心能力：使用 locale -a 列出已生成且可用的语言环境。

教学示例：`locale -a`

- 原用法：`locale -a | sort` — 列出已生成的 locale 并排序，便于查找。
- 参数变化：`locale -a | grep -i utf` — 从可用 locale 中筛选 UTF 相关名称。
- 小任务迁移：`LC_ALL=C locale -a` — 在 C 区域下列出系统可用 locale，避免诊断输出语言差异。

## 217. 确认当前语言环境使用的字符集名称

组：`practice-lesson-v03_34_locale_variables_encoding-04`

核心能力：查询当前或临时指定 locale 的字符映射名称。

教学示例：`locale charmap`

- 原用法：`locale -k charmap` — 以关键字形式读取当前字符映射。
- 参数变化：`LC_ALL=C locale charmap` — 观察 C 区域使用的字符映射，不修改当前 shell 环境。
- 小任务迁移：`LC_ALL=C.UTF-8 locale charmap` — 在已提供 C.UTF-8 的 Ubuntu 环境观察 UTF-8 字符映射。

## 218. 创建名为 dev 的 tmux 新会话，并立即进入该会话

组：`practice-lesson-v03_35_tmux_reptyr_sessions-01`

核心能力：创建命名 tmux 会话并立即连接进入。

教学示例：`tmux new -s dev`

- 原用法：`tmux new -s ops` — 创建名为ops的新会话并立即进入。
- 参数变化：`tmux new -s monitor` — 改变会话名以创建并进入monitor。
- 小任务迁移：`tmux new-session -s backup` — 为备份工作创建命名会话并连接到其中。

## 219. 重新连接到名为 dev 的 tmux 会话

组：`practice-lesson-v03_35_tmux_reptyr_sessions-02`

核心能力：重新连接已有命名 tmux 会话，可使用只读连接。

教学示例：`tmux attach -t dev`

- 原用法：`tmux attach -t ops` — 重新连接实验 ops 会话。
- 参数变化：`tmux attach -t monitor` — 重新连接实验监控会话。
- 小任务迁移：`tmux attach -r -t backup` — 只读连接实验备份会话，避免误输入干扰任务。

## 220. 只读列出当前用户的 tmux 会话，先核对名称、窗口数与连接状态

组：`practice-lesson-v03_35_tmux_reptyr_sessions-03`

核心能力：列出 tmux 会话及其名称、窗口数或连接状态。

教学示例：`tmux list-sessions`

- 原用法：`tmux list-sessions -F '#S: #{session_windows} windows'` — 列出会话名称和窗口数量。
- 参数变化：`tmux list-sessions -F '#S attached=#{session_attached}'` — 列出会话时核对是否已有客户端连接。
- 小任务迁移：`tmux list-sessions -F '#S created=#{session_created}'` — 读取会话创建时间字段以区分长期任务上下文。

## 221. 只读显示当前标准输入连接的终端设备，建立接管前的 TTY 基线

组：`practice-lesson-v03_35_tmux_reptyr_sessions-09`

核心能力：检查标准输入是否为终端，并可显示或提取终端设备名。

教学示例：`tty`

- 原用法：`tty < /dev/tty` — 从当前控制终端取得标准输入并显示终端设备。
- 参数变化：`tty -s` — 通过退出状态只检查标准输入是否为终端。
- 小任务迁移：`tty | sed 's#^/dev/##'` — 读取标准输入终端名后去掉设备目录前缀。

## 222. 只读确认当前 shell 能否从 PATH 找到 reptyr

组：`practice-lesson-v03_35_tmux_reptyr_sessions-11`

核心能力：从 PATH 定位指定外部工具并确认是否安装。

教学示例：`command -v reptyr`

- 原用法：`command -v python3` — 从当前PATH定位Python外部程序。
- 参数变化：`which tar` — 使用which定位归档工具的外部路径。
- 小任务迁移：`command -v curl` — 在运行网络检查前确认curl能否从PATH找到。

## 223. 统计 /etc/passwd 的行数、单词数和字节数

组：`practice-lesson-wc-01`

核心能力：通过 wc 统计行、字节或单词数量。

教学示例：`wc /etc/passwd`

- 原用法：`wc -l ~/ops-lab/report.txt` — 使用 wc -l 按换行符统计报告行数，不改为字节或词数。
- 参数变化：`wc -c ~/ops-lab/report.txt` — 统计实验报告字节数，和字符数区分。
- 小任务迁移：`wc -w ~/ops-lab/notes.txt` — 按空白分隔统计实验笔记词数。

## 224. 统计系统中有多少个用户账户

组：`practice-lesson-wc-02`

核心能力：使用 wc -l 统计文本行数。

教学示例：`wc -l /etc/passwd`

- 原用法：`wc -l ~/ops-lab/report.txt` — 使用 wc -l 按换行符统计报告行数，不改为字节或词数。
- 参数变化：`wc -l ~/ops-lab/settings.ini` — 保留 wc -l 行数统计，改变目标以核对配置文件规模。
- 小任务迁移：`wc -l ~/ops-lab/notes.txt ~/ops-lab/report.txt` — 分别统计两份实验文件行数和合计。

## 225. 下载文件到当前目录

组：`practice-lesson-wget-01`

核心能力：通过 wget 下载响应正文到本地文件。

教学示例：`wget -O hello-source.tar.gz https://ftp.gnu.org/gnu/hello/hello-2.12.2.tar.gz`

- 原用法：`wget -O example.html https://example.com/` — 下载示例网页并指定本地输出文件名。
- 参数变化：`wget --timeout=5 --tries=1 https://example.com/` — 限制下载等待与重试次数，避免持续等待。
- 小任务迁移：`wget -O hello.tar.gz https://ftp.gnu.org/gnu/hello/hello-2.12.2.tar.gz` — 从 GNU 官方站点下载实验归档并指定本地名称。

## 226. 删除当前目录下所有 .tmp 文件

组：`practice-lesson-xargs-01`

核心能力：用 find 的 NUL 边界配合 xargs -0 -r rm 删除已确认临时文件。

教学示例：`find . -type f -name '*.tmp' -print0 | xargs -0 -r rm --`

- 原用法：`find ~/ops-lab/cache -type f -name '*.tmp' -print0 | xargs -0 -r rm --` — 按NUL边界删除课程已确认可丢弃的临时文件。
- 参数变化：`find ~/ops-lab/cache -maxdepth 1 -type f -name '*.tmp' -print0 | xargs -0 -r rm --` — 只在缓存第一层选择临时文件并保持安全文件名边界。
- 小任务迁移：`find ~/ops-lab/cache -type f -name '*.cache' -print0 | xargs -0 -r rm -v --` — 删除已确认可再生缓存并显示每个目标，空列表不执行rm。

## 227. 管道

组：`practice-symbol-pipe_redirect-pipe`

核心能力：用单个管道把一个程序的标准输出交给另一个程序。

教学示例：`ls -la /etc | head -5`

- 原用法：`ls /var/log | head -n 3` — 把日志文件列表通过管道交给 head，保留前三行。
- 参数变化：`printf '%s\n' alpha beta alpha | sort` — 把三行实验文本送给 sort 排序。
- 小任务迁移：`grep 'WARN' backend.log | wc -l` — 把匹配的警告行交给 wc 统计。

## 228. 输出重定向（覆盖）

组：`practice-symbol-pipe_redirect-redirect_stdout_overwrite`

核心能力：用 > 覆盖写入标准输出，区别于追加。

教学示例：`echo "Hello, Linux!" > greeting.txt`

- 原用法：`printf 'hello\n' > ~/ops-lab/greeting.txt` — 把实验问候写入文件，已有内容会被覆盖。
- 参数变化：`ls ~/ops-lab > ~/ops-lab/inventory.txt` — 把实验目录列表保存为清单。
- 小任务迁移：`date -u > ~/ops-lab/checked-at.txt` — 保存 UTC 检查时间到实验文件。

## 229. 输出重定向（追加）

组：`practice-symbol-pipe_redirect-redirect_stdout_append`

核心能力：用 >> 在保留旧内容的前提下追加标准输出。

教学示例：`echo "第一行" > notes.txt && echo "第二行" >> notes.txt && cat notes.txt`

- 原用法：`printf 'started\n' >> ~/ops-lab/deploy.log` — 向实验部署日志追加启动标记。
- 参数变化：`date -u >> ~/ops-lab/deploy.log` — 在实验日志末尾追加 UTC 时间。
- 小任务迁移：`printf 'finished\n' >> ~/ops-lab/deploy.log` — 追加结束标记，保留之前内容。

## 230. 标准错误重定向

组：`practice-symbol-pipe_redirect-redirect_stderr`

核心能力：用 2> 单独保存标准错误。

教学示例：`ls /nonexistent 2> errors.txt ↵ cat errors.txt`

- 原用法：`ls ~/ops-lab/missing 2> ~/ops-lab/errors.txt` — 只把查找缺失路径的标准错误保存到文件。
- 参数变化：`find /var/log -type f 2> ~/ops-lab/find-errors.txt` — 保存遍历权限错误，正常路径仍在标准输出。
- 小任务迁移：`python3 ~/ops-lab/check.py 2> ~/ops-lab/python-errors.txt` — 把实验脚本异常输出单独记录。

## 231. 合并标准错误到标准输出

组：`practice-symbol-pipe_redirect-redirect_stderr_to_stdout`

核心能力：用 2>&1 将标准错误复制到当时的标准输出目标。

教学示例：`ls /etc /nonexistent 2>&1 | grep -i "cannot"`

- 原用法：`ls /etc /missing 2>&1 | head -n 5` — 先合并标准错误到标准输出，再经管道截取前五行。
- 参数变化：`python3 ~/ops-lab/check.py > ~/ops-lab/run.log 2>&1` — 先指定日志文件，再让标准错误跟随当前标准输出。
- 小任务迁移：`ls ~/ops-lab/missing 2>&1 | wc -l` — 合并错误输出后统计诊断消息行数。

## 232. 输入重定向

组：`practice-symbol-pipe_redirect-redirect_stdin`

核心能力：用 < 将文件接入标准输入。

教学示例：`wc -l < /etc/passwd`

- 原用法：`wc -l < ~/ops-lab/notes.txt` — 通过标准输入统计实验笔记行数。
- 参数变化：`sort < ~/ops-lab/hosts.txt` — 从重定向输入读取实验主机列表并排序。
- 小任务迁移：`python3 -m json.tool < ~/ops-lab/response.json` — 通过标准输入验证和格式化实验 JSON。

## 233. Here Document

组：`practice-symbol-pipe_redirect-heredoc`

核心能力：使用成对分隔词给程序提供 Here Document 标准输入。

教学示例：`cat << EOF ↵ Hello, $USER! ↵ Today is $(date +%A). ↵ EOF`

- 原用法：`cat <<'TEXT' ↵ hello ↵ TEXT` — 输入带引号结束标识的 here-document，正文不做变量展开。
- 参数变化：`cat <<'CONFIG' ↵ port=8000 ↵ CONFIG` — 用 here-document 演示一行实验配置。
- 小任务迁移：`cat <<'VARS' ↵ $HOME ↵ VARS` — 观察带引号结束标识保护正文中的变量名。

## 234. tee 分流器

组：`practice-symbol-pipe_redirect-tee`

核心能力：用 tee 同时输出到终端与文件，可选择追加。

教学示例：`df -h | tee disk_usage.txt`

- 原用法：`date -u | tee ~/ops-lab/checked-at.txt` — 同时显示 UTC 时间并写入实验文件。
- 参数变化：`printf 'ready\n' | tee -a ~/ops-lab/deploy.log` — 同时显示状态并追加日志。
- 小任务迁移：`df -h / | tee ~/ops-lab/disk-report.txt` — 把根文件系统容量报告同时显示和保存。

## 235. 单引号

组：`practice-symbol-quotes_escape-single_quote`

核心能力：用单引号把变量标记或含空格文本作为字面参数。

教学示例：`echo '$HOME'`

- 原用法：`printf '%s\n' '$USER'` — 单引号保护美元符，输出变量名文本。
- 参数变化：`printf '%s\n' 'cost=$20'` — 单引号让价格文本不发生变量展开。
- 小任务迁移：`printf '%s\n' 'hello ops team'` — 单引号把有空格的文本作为一个参数。

## 236. 双引号

组：`practice-symbol-quotes_escape-double_quote`

核心能力：在双引号中展开变量或命令替换，同时保留一个参数的边界。

教学示例：`echo "$HOME"`

- 原用法：`printf '%s\n' "$USER"` — 输出当前 USER 环境值。
- 参数变化：`printf '%s\n' "home=$HOME"` — 把固定前缀与变量内容组合为一个参数。
- 小任务迁移：`printf '%s\n' "kernel=$(uname -r)"` — 双引号内允许命令替换，并保持结果整体。

## 237. 反斜杠（转义符）

组：`practice-symbol-quotes_escape-backslash`

核心能力：使用反斜杠保护美元符、空格或引号的字面含义。

教学示例：`echo "price is \$100"`

- 原用法：`printf '%s\n' cost=\$20` — 用反斜杠保护美元符，避免参数展开。
- 参数变化：`ls ~/ops-lab/file\ with\ spaces.txt` — 用反斜杠保护路径中的空格。
- 小任务迁移：`printf '%s\n' "she said \"hello\""` — 在双引号中用反斜杠保留字面双引号。

## 238. 反引号（旧式命令替换）

组：`practice-symbol-quotes_escape-backtick`

核心能力：使用成对反引号捕获子命令输出。

教学示例：``echo `date```

- 原用法：``printf '%s\n' `whoami``` — 读取传统反引号命令替换结果，新脚本通常优先用 $()。
- 参数变化：``printf '%s\n' "date=`date +%F`"`` — 在双引号中使用传统日期命令替换。
- 小任务迁移：``printf '%s\n' "host=`hostname`"`` — 把传统命令替换和固定文本组合。

## 239. 命令替换

组：`practice-symbol-quotes_escape-command_substitution`

核心能力：使用 $(...) 捕获命令输出并嵌入参数。

教学示例：`echo $(whoami)`

- 原用法：`printf '%s\n' "$(hostname)"` — 通过命令替换获取主机名，并用双引号保护。
- 参数变化：`printf '%s\n' "today=$(date +%F)"` — 把日期命令输出嵌入实验提示。
- 小任务迁移：`printf '%s\n' "cpus=$(nproc)"` — 把可用 CPU 数嵌入状态文本。

## 240. 点号元字符

组：`practice-symbol-regex_basics-regex_dot`

核心能力：正则中的非转义点号匹配任意单个字符。

教学示例：`echo -e 'hat\nhit\nhot\nhut\nht' | grep 'h.t'`

- 原用法：`printf 'cat\ncut\ncoat\n' | grep '^c.t$'` — 正则点匹配单个字符，锚点限制整行。
- 参数变化：`printf 'log\nlag\nlg\n' | grep '^l.g$'` — 比较单字符位置缺失时是否匹配。
- 小任务迁移：`grep '5.0' status.txt` — 在实验状态文本中练习点匹配任意一个字符，不是字面小数点。

## 241. 星号量词

组：`practice-symbol-regex_basics-regex_star`

核心能力：正则星号将前一元素重复零次或多次。

教学示例：`echo -e 'ac\nabc\nabbc\nabbbc' | grep 'ab*c'`

- 原用法：`printf 'ac\nabc\nabbc\n' | grep '^ab*c$'` — 星号重复前面的 b 零次或多次。
- 参数变化：`grep 'ERROR.*timeout' backend.log` — 点星组合匹配错误与 timeout 之间任意长度文本。
- 小任务迁移：`printf 'ab\nabb\nac\n' | grep '^abb*$'` — 观察星号只作用于紧邻的前一个 b。

## 242. 加号量词

组：`practice-symbol-regex_basics-regex_plus`

核心能力：扩展正则加号要求前一元素至少出现一次。

教学示例：`echo -e 'ggle\ngogle\ngoogle\ngooogle' | grep -E 'go+gle'`

- 原用法：`printf 'ac\nabc\nabbc\n' | grep -E '^ab+c$'` — 扩展正则加号要求 b 至少出现一次。
- 参数变化：`grep -E '^[0-9]+$' ports.txt` — 只选择完全由一位或多位数字组成的行。
- 小任务迁移：`grep -E 'WARN[[:space:]]+' backend.log` — 匹配 WARN 后至少一个空白字符。

## 243. 问号量词

组：`practice-symbol-regex_basics-regex_question`

核心能力：扩展正则问号使前一元素出现零次或一次。

教学示例：`echo -e 'color\ncolour\ncolouur' | grep -E 'colou?r'`

- 原用法：`printf 'color\ncolour\ncolouur\n' | grep -E '^colou?r$'` — 问号让 u 可出现零次或一次。
- 参数变化：`grep -E '^https?://' urls.txt` — 同时匹配 HTTP 和 HTTPS 行首协议。
- 小任务迁移：`grep -E '^-?[0-9]+$' numbers.txt` — 让负号可选，后面至少一位数字。

## 244. 脱字符（行首锚点）

组：`practice-symbol-regex_basics-regex_caret`

核心能力：使用正则 ^ 将匹配约束在行首。

教学示例：`grep '^root' /etc/passwd`

- 原用法：`grep '^root:' /etc/passwd` — 只选择以 root: 开头的账号记录。
- 参数变化：`grep '^#' ~/ops-lab/settings.ini` — 选出从行首开始的注释。
- 小任务迁移：`grep '^ERROR' backend.log` — 只找以 ERROR 开头的实验日志。

## 245. 美元符号（行尾锚点）

组：`practice-symbol-regex_basics-regex_dollar`

核心能力：使用正则 $ 将匹配约束在行尾。

教学示例：`grep 'bash$' /etc/passwd`

- 原用法：`grep '/bash$' /etc/passwd` — 只匹配登录 shell 以 /bash 结尾的账号记录。
- 参数变化：`grep '\.log$' filenames.txt` — 选择以字面 .log 结尾的文件名行。
- 小任务迁移：`grep 'done$' status.txt` — 只匹配末尾为 done 的实验状态行。

## 246. 方括号字符类

组：`practice-symbol-regex_basics-regex_bracket`

核心能力：使用正则方括号选择字符集合或范围。

教学示例：`echo -e 'apple\nbanana\norange\numbrella' | grep '[aeiou]'`

- 原用法：`grep '^[0-9]' ports.txt` — 匹配第一字符是数字的行。
- 参数变化：`grep '[A-Z]' labels.txt` — 匹配包含大写拉丁字母的实验标签。
- 小任务迁移：`grep '[^0-9]' ports.txt` — 找出包含非数字字符的行，和全数字行区别。

## 247. 圆括号分组

组：`practice-symbol-regex_basics-regex_group`

核心能力：使用扩展正则圆括号对候选或重复单元分组。

教学示例：`echo -e 'ab\nabab\nababab\naabb' | grep -E '^(ab)+$'`

- 原用法：`grep -E '(ERROR|WARN)' backend.log` — 分组和或匹配两类日志等级。
- 参数变化：`printf 'ab\nabab\naabb\n' | grep -E '^(ab)+$'` — 加号作用于整个 ab 分组。
- 小任务迁移：`grep -E '^(GET|POST) ' requests.txt` — 只匹配以 GET 或 POST 加空格开头的请求行。

## 248. 分号（命令分隔符）

组：`practice-symbol-special_chars-semicolon`

核心能力：用分号顺序执行命令，不根据前一条成败短路。

教学示例：`echo "hello"; echo "world"`

- 原用法：`printf 'one\n'; printf 'two\n'` — 分号顺序运行两个独立输出动作。
- 参数变化：`false; printf 'still runs\n'` — 观察前一命令失败也不阻止分号后的动作。
- 小任务迁移：`pwd; date -u` — 顺序观察当前目录和 UTC 时间。

## 249. 逻辑与（AND）

组：`practice-symbol-special_chars-logical_and`

核心能力：用 && 只在左侧成功时继续下一动作。

教学示例：`mkdir newdir && cd newdir && pwd`

- 原用法：`test -f ~/ops-lab/settings.ini && cat ~/ops-lab/settings.ini` — 只有文件存在时才读取实验配置。
- 参数变化：`true && printf 'success\n'` — 成功状态触发与操作符后的动作。
- 小任务迁移：`test -d ~/ops-lab && ls ~/ops-lab` — 仅当实验目录存在时列出内容。

## 250. 逻辑或（OR）

组：`practice-symbol-special_chars-logical_or`

核心能力：用 || 在左侧失败时执行备用动作。

教学示例：`cd /nonexistent || echo "目录不存在"`

- 原用法：`test -f ~/ops-lab/settings.ini || printf 'missing config\n'` — 文件不存在时输出诊断提示。
- 参数变化：`false || printf 'fallback\n'` — 失败状态触发或操作符后的后备动作。
- 小任务迁移：`command -v tcpdump || printf 'install tcpdump first\n'` — 缺少实验抓包工具时给出明确提示。

## 251. 后台执行符

组：`practice-symbol-special_chars-background`

核心能力：用末尾 & 启动后台作业。

教学示例：`sleep 100 &`

- 原用法：`sleep 15 &` — 把短实验等待任务放到后台，shell 立即可继续输入。
- 参数变化：`sleep 25 > ~/ops-lab/sleep.log 2>&1 &` — 先重定向输出，再让实验任务后台运行。
- 小任务迁移：`nohup sleep 30 > ~/ops-lab/nohup.log 2>&1 &` — 演练忽略 SIGHUP 并明确日志去向的后台任务。

## 252. 感叹号（历史展开与逻辑非）

组：`practice-symbol-special_chars-exclamation`

核心能力：区分 Bash 感叹号的历史展开和命令状态取反。

教学示例：`!!`

- 原用法：`!!:p` — 在启用历史展开的交互式 Bash 中只显示上一条历史命令，不执行。
- 参数变化：`!-2:p` — 在交互式 Bash 中只查看倒数第二条历史命令的展开结果。
- 小任务迁移：`! test -f ~/ops-lab/missing` — 对实验文件存在测试取反，和历史展开的感叹号区别。

## 253. 井号（注释符）

组：`practice-symbol-special_chars-comment`

核心能力：使用未被引用的 # 开始 shell 注释。

教学示例：`echo hello # 这是一条注释`

- 原用法：`printf 'ready\n' # readiness marker` — 将就绪标记后的文本作为注释而不执行。
- 参数变化：`# inspect the service before restarting` — 整行注释记录检查目的，不运行任何命令。
- 小任务迁移：`printf '%s\n' '# literal text' # quoted hash is data` — 区分引号内的字面井号与引号外真正开始的注释。

## 254. 波浪号（主目录快捷方式）

组：`practice-symbol-special_chars-tilde`

核心能力：使用未被引用的 ~ 展开当前用户主目录。

教学示例：`cd ~`

- 原用法：`ls ~/ops-lab` — 未引用的波浪号在词首展开为当前用户家目录。
- 参数变化：`cd ~/ops-lab/input` — 通过家目录展开进入实验输入目录。
- 小任务迁移：`printf '%s\n' ~` — 把波浪号展开结果打印出来。

## 255. 点号（source 命令 / 当前目录）

组：`practice-symbol-special_chars-dot_command`

核心能力：用 . 或 source 在当前 shell 加载可信脚本。

教学示例：`. ~/.bashrc`

- 原用法：`. ~/ops-lab/env.sh` — 在当前 shell 读取可信实验脚本，使其变量在当前会话生效。
- 参数变化：`source ~/ops-lab/env.sh` — 使用 Bash source 写法读取同一实验环境脚本。
- 小任务迁移：`. .venv/bin/activate` — 在当前 shell 激活实验虚拟环境，点命令不能由子进程代替。

## 256. 变量引用

组：`practice-symbol-variables-var_reference`

核心能力：展开变量值并用引号保留参数边界。

教学示例：`echo $HOME`

- 原用法：`printf '%s\n' "$USER"` — 输出当前 USER 环境值。
- 参数变化：`printf '%s\n' "$PATH"` — 读取命令搜索路径变量。
- 小任务迁移：`printf '%s\n' "$SHELL"` — 输出登录 shell 环境值，它不一定是当前解释器。

## 257. 花括号变量引用

组：`practice-symbol-variables-var_brace`

核心能力：用 ${NAME} 划定变量名称边界并拼接前后文字。

教学示例：`echo "${HOME}/docs"`

- 原用法：`printf '%s\n' "${USER}_logs"` — 使用花括号分隔变量名与后缀。
- 参数变化：`printf '%s\n' "${HOME}/ops-lab"` — 把家目录变量和实验子路径组合。
- 小任务迁移：`printf '%s\n' "${LANG}:encoding"` — 使用花括号标识变量展开边界。

## 258. 退出状态码

组：`practice-symbol-variables-exit_status`

核心能力：紧跟待检查命令读取 $? 退出状态。

教学示例：`ls /etc > /dev/null && echo $?`

- 原用法：`true; printf '%s\n' "$?"` — 观察成功命令退出状态。
- 参数变化：`false; printf '%s\n' "$?"` — 观察失败命令退出状态。
- 小任务迁移：`test -f ~/ops-lab/settings.ini; printf '%s\n' "$?"` — 读取实验文件测试的状态，打印前不要插入其他命令。

## 259. 脚本/Shell 名称

组：`practice-symbol-variables-script_name`

核心能力：读取 $0 以识别 shell 或显式指定的 bash -c 调用名。

教学示例：`echo $0`

- 原用法：`bash -c 'printf "%s\n" "$0"' monitor` — 把 monitor 作为 Bash -c 的零号参数。
- 参数变化：`bash -c 'printf "%s\n" "$0"' backup` — 比较另一个实验脚本标识。
- 小任务迁移：`printf 'shell=%s\n' "$0"` — 观察当前 shell 的启动名称。

## 260. 参数个数

组：`practice-symbol-variables-arg_count`

核心能力：读取 $# 计算按引用边界传入的位置参数个数。

教学示例：`bash -c 'echo $#' _ a b c`

- 原用法：`bash -c 'printf "%s\n" "$#"' demo one` — 计算一个位置参数，demo 被用作 $0。
- 参数变化：`bash -c 'printf "%s\n" "$#"' demo one two` — 读取 $# 计得两个位置参数；demo 是 $0，不计入参数个数。
- 小任务迁移：`bash -c 'printf "%s\n" "$#"' demo 'one two'` — 引号中的空格不把参数拆成两个。

## 261. 所有位置参数

组：`practice-symbol-variables-all_args`

核心能力：使用带双引号的 "$@" 保留每个位置参数的边界。

教学示例：`bash -c 'echo "$@"' _ hello world`

- 原用法：`bash -c 'printf "<%s>\n" "$@"' demo alpha beta` — 逐项保留两个位置参数。
- 参数变化：`bash -c 'printf "<%s>\n" "$@"' demo 'alpha beta' gamma` — 观察双引号 $@ 保留参数中的空格。
- 小任务迁移：`bash -c 'for item in "$@"; do printf "%s\n" "$item"; done' demo one two` — 安全遍历实验位置参数。

## 262. 当前进程 ID

组：`practice-symbol-variables-current_pid`

核心能力：读取 $$ 获取当前 shell 的 PID 或用于相关查询。

教学示例：`echo $$`

- 原用法：`printf 'shell pid=%s\n' "$$"` — 读取当前 shell 的进程标识。
- 参数变化：`ps -p $$ -o pid,ppid,tty` — 用当前 shell PID 查询进程关系。
- 小任务迁移：`printf '/tmp/ops-%s.log\n' "$$"` — 把 shell PID 加入实验文件名；临时敏感文件仍应使用 mktemp。

## 263. 后台进程 PID

组：`practice-symbol-variables-last_bg_pid`

核心能力：启动后台作业后读取 $! 获取最后后台 PID。

教学示例：`sleep 100 & echo $!`

- 原用法：`sleep 5 & printf 'pid=%s\n' "$!"` — 后台启动短任务并立即读取其 PID。
- 参数变化：`sleep 7 & wait "$!"` — 等待最近的后台实验任务完成。
- 小任务迁移：`sleep 9 & ps -p "$!" -o pid,args` — 用后台 PID 查询实验等待进程。

## 264. 星号通配符

组：`practice-symbol-wildcards-asterisk`

核心能力：使用未引用的 * 对文件名进行任意长度通配。

教学示例：`ls *.txt`

- 原用法：`ls ~/ops-lab/*.log` — 由 shell 展开实验目录中所有 .log 文件。
- 参数变化：`ls ~/ops-lab/report*` — 匹配实验报告名前缀，星号可匹配零个字符。
- 小任务迁移：`cp ~/ops-lab/input/*.txt ~/ops-lab/backup/` — 把匹配的实验文本文件复制到备份目录。

## 265. 问号通配符

组：`practice-symbol-wildcards-question_mark`

核心能力：使用未引用的 ? 匹配文件名中的单个字符。

教学示例：`ls file?.txt`

- 原用法：`ls ~/ops-lab/log?.txt` — 问号匹配实验文件名中恰好一个字符。
- 参数变化：`ls ~/ops-lab/log??.txt` — 两个问号要求两个字符位置。
- 小任务迁移：`ls ~/ops-lab/day?.csv` — 匹配一位日期标记的实验 CSV。

## 266. 方括号字符类

组：`practice-symbol-wildcards-bracket_class`

核心能力：使用文件名方括号通配一个字符集合或范围。

教学示例：`ls file[0-9].txt`

- 原用法：`ls ~/ops-lab/log[0-9].txt` — 方括号只匹配一个数字位置。
- 参数变化：`ls ~/ops-lab/[Rr]eport.txt` — 匹配实验报告首字母的两种大小写。
- 小任务迁移：`ls ~/ops-lab/*.[ch]` — 匹配实验 C 源文件或头文件扩展名。

## 267. 花括号展开

组：`practice-symbol-wildcards-brace_expansion`

核心能力：使用花括号列表或序列在 shell 中生成多个词。

教学示例：`echo {1..5}`

- 原用法：`printf '%s\n' log-{1..3}.txt` — 花括号在命令运行前生成三个实验文件名。
- 参数变化：`mkdir -p ~/ops-lab/{logs,cache,reports}` — 用列表展开创建三个实验目录。
- 小任务迁移：`printf '%s\n' report.{csv,json}` — 用后缀列表生成两种实验报告名称。

## 268. 单中括号条件测试

组：`practice-symbol-tests_conditionals-single-bracket-test`

核心能力：用 [ ... ] 进行条件测试，并保持操作数和括号分隔。

教学示例：`(mode=safe; [ "$mode" = "safe" ] && printf 'safe\n')`

- 原用法：`[ -d ~/ops-lab ] && printf 'directory exists\n'` — 方括号命令的各参数需要空格，成功时显示提示。
- 参数变化：`[ -r ~/ops-lab/settings.ini ] && printf 'readable\n'` — 检查实验配置对当前用户是否可读。
- 小任务迁移：`[ -n "$USER" ] && printf 'user is set\n'` — 双引号保护变量，再测试字符串非空。

## 269. 双中括号条件表达式

组：`practice-symbol-tests_conditionals-double-bracket-test`

核心能力：使用 Bash [[ ... ]] 条件表达式进行模式或属性判断。

教学示例：`(name=server.log; [[ $name == *.log ]] && printf 'log file\n')`

- 原用法：`[[ report.log == *.log ]] && printf 'log suffix\n'` — Bash 双括号右侧未引用模式用于通配匹配。
- 参数变化：`[[ 8080 =~ ^[0-9]+$ ]] && printf 'numeric\n'` — Bash 正则测试检查实验端口文本格式。
- 小任务迁移：`[[ -d ~/ops-lab && -r ~/ops-lab/settings.ini ]] && printf 'ready\n'` — 双括号内组合两个检查，满足后才提示。

## 270. 文件属性测试

组：`practice-symbol-tests_conditionals-file-tests`

核心能力：使用 test 的文件属性操作符判断类型或权限。

教学示例：`test -f .env && echo exists`

- 原用法：`test -f ~/ops-lab/report.txt && printf 'regular file\n'` — 确认实验报告是普通文件。
- 参数变化：`test -d ~/ops-lab/backup && printf 'backup directory\n'` — 确认实验备份目录存在。
- 小任务迁移：`test -x ~/ops-lab/check.sh && printf 'executable\n'` — 确认实验脚本对当前用户可执行。

## 271. 字符串测试

组：`practice-symbol-tests_conditionals-string-tests`

核心能力：使用字符串空值、非空或相等测试。

教学示例：`[ -z "$optional_tag" ] && printf 'tag missing\n'`

- 原用法：`[ -z "$optional_label" ] && printf 'label missing\n'` — 测试可选实验标签是否为空。
- 参数变化：`[ -n "$HOME" ] && printf 'home present\n'` — 测试 HOME 是否为非空字符串。
- 小任务迁移：`[ "$USER" = ace ] && printf 'lab user\n'` — 按字符串比较当前用户变量与实验账号名。

## 272. 整数比较测试

组：`practice-symbol-tests_conditionals-numeric-tests`

核心能力：用 test 的整数比较运算符而非字符串排序比较数值。

教学示例：`(retries=3; [ "$retries" -ge 3 ] && printf 'retry limit reached\n')`

- 原用法：`[ 3 -gt 1 ] && printf 'greater\n'` — 用整数比较运算测试 3 大于 1。
- 参数变化：`[ 8080 -le 65535 ] && printf 'below upper bound\n'` — 检查实验端口不超过上限，完整验证还要检查下限。
- 小任务迁移：`[ 0 -eq 0 ] && printf 'equal\n'` — 用 -eq 比较整数相等。

## 273. 成功后继续

组：`practice-symbol-tests_conditionals-logical-and-flow`

核心能力：用 && 只在左侧成功时继续下一动作。

教学示例：`test -f report.txt && test -r report.txt && printf 'ready to inspect\n'`

- 原用法：`test -f ~/ops-lab/settings.ini && cat ~/ops-lab/settings.ini` — 只有文件存在时才读取实验配置。
- 参数变化：`true && printf 'success\n'` — 成功状态触发与操作符后的动作。
- 小任务迁移：`test -d ~/ops-lab && ls ~/ops-lab` — 仅当实验目录存在时列出内容。

## 274. 失败后走备用分支

组：`practice-symbol-tests_conditionals-logical-or-flow`

核心能力：用 || 在左侧失败时执行备用动作。

教学示例：`test -f settings.ini || printf 'settings missing\n'`

- 原用法：`test -f ~/ops-lab/settings.ini || printf 'missing config\n'` — 文件不存在时输出诊断提示。
- 参数变化：`false || printf 'fallback\n'` — 失败状态触发或操作符后的后备动作。
- 小任务迁移：`command -v tcpdump || printf 'install tcpdump first\n'` — 缺少实验抓包工具时给出明确提示。

## 275. 圆括号子 Shell 分组

组：`practice-symbol-grouping_expansion-subshell-group`

核心能力：用圆括号在子 shell 中组合命令并隔离状态。

教学示例：`( cd /srv/app && pwd ); pwd`

- 原用法：`( cd /tmp; pwd )` — 在子 shell 内切换目录，返回后不改变父 shell 工作目录。
- 参数变化：`( printf 'one\n'; printf 'two\n' )` — 在同一个子 shell 分组中执行两个输出动作。
- 小任务迁移：`( umask 077; umask )` — 仅在子 shell 中调整并读取掩码，父 shell 不受影响。

## 276. 花括号当前 Shell 分组

组：`practice-symbol-grouping_expansion-current-shell-group`

核心能力：用花括号在当前 shell 组合命令，并在 } 前保留分号。

教学示例：`{ printf 'alpha\n'; printf 'beta\n'; }`

- 原用法：`{ printf 'one\n'; printf 'two\n'; }` — 花括号在当前 shell 分组，结束前需要分号和空格。
- 参数变化：`{ pwd; date -u; }` — 在当前 shell 顺序输出目录与时间。
- 小任务迁移：`{ printf 'ready\n'; } > ~/ops-lab/group.log` — 把整个当前 shell 分组的输出重定向到实验文件。

## 277. 命令替换

组：`practice-symbol-grouping_expansion-command-substitution`

核心能力：使用 $(...) 捕获命令输出并嵌入参数。

教学示例：`printf 'cpus=%s\n' "$(nproc)"`

- 原用法：`printf '%s\n' "$(hostname)"` — 通过命令替换获取主机名，并用双引号保护。
- 参数变化：`printf '%s\n' "today=$(date +%F)"` — 把日期命令输出嵌入实验提示。
- 小任务迁移：`printf '%s\n' "cpus=$(nproc)"` — 把可用 CPU 数嵌入状态文本。

## 278. 参数展开边界

组：`practice-symbol-grouping_expansion-parameter-expansion-basic`

核心能力：用 ${NAME} 划定变量名称边界并拼接前后文字。

教学示例：`base=report; printf '%s\n' "${base}_old"`

- 原用法：`printf '%s\n' "${USER}_logs"` — 使用花括号分隔变量名与后缀。
- 参数变化：`printf '%s\n' "${HOME}/ops-lab"` — 把家目录变量和实验子路径组合。
- 小任务迁移：`printf '%s\n' "${LANG}:encoding"` — 使用花括号标识变量展开边界。

## 279. 参数默认值与变换

组：`practice-symbol-grouping_expansion-parameter-expansion-operators`

核心能力：使用参数展开的默认值或后缀变换操作符。

教学示例：`printf '%s\n' "${APP_MODE:-development}"`

- 原用法：`printf '%s\n' "${APP_PORT:-8000}"` — 变量未设置或为空时使用实验默认端口，不修改变量。
- 参数变化：`printf '%s\n' "${LOG_LEVEL:-info}"` — 为实验日志等级提供缺省值。
- 小任务迁移：`file=report.csv; printf '%s\n' "${file%.csv}.json"` — 删除匹配的短后缀并组合新扩展名。

## 280. 花括号展开

组：`practice-symbol-grouping_expansion-brace-expansion`

核心能力：使用花括号列表或序列在 shell 中生成多个词。

教学示例：`printf '%s\n' report.{txt,csv}`

- 原用法：`printf '%s\n' log-{1..3}.txt` — 花括号在命令运行前生成三个实验文件名。
- 参数变化：`mkdir -p ~/ops-lab/{logs,cache,reports}` — 用列表展开创建三个实验目录。
- 小任务迁移：`printf '%s\n' report.{csv,json}` — 用后缀列表生成两种实验报告名称。

## 281. 展开顺序与引用边界

组：`practice-symbol-grouping_expansion-expansion-order`

核心能力：比较引用如何保留参数边界或阻止变量展开。

教学示例：`items='alpha beta'; printf '<%s>\n' "$items"`

- 原用法：`words='one two'; printf '<%s>\n' "$words"` — 双引号保护展开结果，保留一个包含空格的参数。
- 参数变化：`words='one two'; printf '<%s>\n' $words` — 未引用展开经历分词，本例变成两个参数。
- 小任务迁移：`printf '%s\n' '$USER'` — 单引号保护美元符，输出变量名文本。
