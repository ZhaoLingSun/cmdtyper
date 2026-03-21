# Socrates Vague Token Description Audit — 2026-03-21

**Auditor:** Professor Socrates  
**Scope:** `/home/ace/workspaces/cmdtyper` — all lesson files (`data/lessons/*.toml`) and command files (`data/commands/*.toml`)  
**Date:** 2026-03-21

---

## 1. Summary

**Total vague descriptions found: 113**

Breakdown by file:

| File | Vague Count |
|------|------------|
| data/lessons/xargs.toml | 12 |
| data/lessons/sort.toml | 8 |
| data/lessons/top.toml | 7 |
| data/lessons/ip.toml | 12 |
| data/lessons/ping.toml | 8 |
| data/lessons/awk.toml | 3 |
| data/lessons/sed.toml | 3 |
| data/lessons/mv.toml | 6 |
| data/lessons/cp.toml | 7 |
| data/lessons/rm.toml | 4 |
| data/lessons/uniq.toml | 4 |
| data/lessons/git.toml | 5 |
| data/lessons/kill.toml | 4 |
| data/lessons/docker.toml | 7 |
| data/lessons/ssh.toml | 6 |
| data/lessons/wc.toml | 3 |
| data/lessons/head.toml | 3 |
| data/lessons/systemctl.toml | 2 |
| data/lessons/tail.toml | 2 |
| data/lessons/tar.toml | 2 |
| data/lessons/nginx.toml | 3 |
| data/lessons/mkdir.toml | 2 |
| data/lessons/chmod.toml | 2 |
| data/lessons/ps.toml | 4 |
| data/commands/ (all files) | 0 |

**Overall verdict: FAIL — 113 vague descriptions remain across 24 lesson files.**

---

## 2. Lesson Files — Detailed Issues

### data/lessons/awk.toml

| Command | Token | Vague Pattern |
|---------|-------|---------------|
| `awk '/ERROR/ {count++} END {print "Total errors:", count}' app.log` | `/ERROR/ {count++} END {print "Total errors:", count}` | 这是文本处理表达式，用来定义匹配规则和输出/替换动作 |
| `awk -F: 'BEGIN {print "User\tUID\tShell"} $3>=1000 && $3<65534 {print $1"\t"$3"\t"$7}' /etc/passwd` | `BEGIN {print "User\tUID\tShell"} $3>=1000 && $3<65534 {print $1"\t"$3"\t"$7}` | 这是文本处理表达式，用来定义匹配规则和输出/替换动作 |
| `awk '{sum+=$1; count++} END {printf "Average: %.2f\n", sum/count}' numbers.txt` | `{sum+=$1; count++} END {printf "Average: %.2f\n", sum/count}` | 这是文本处理表达式，用来定义匹配规则和输出/替换动作 |

**Note:** Each of these awk programs is distinct and deserves a specific explanation of what it does (count errors, filter /etc/passwd users, compute average), not a generic pattern label.

---

### data/lessons/chmod.toml

| Command | Token | Vague Pattern |
|---------|-------|---------------|
| `chmod 644 index.html` | `index.html` | 输入或输出文件名，命令会对这个文件进行读取/写入/处理 |
| `chmod -v go-rwx secret.key` | `secret.key` | 输入或输出文件名，命令会对这个文件进行读取/写入/处理 |

---

### data/lessons/cp.toml

| Command | Token | Vague Pattern |
|---------|-------|---------------|
| `cp -a /etc/nginx /backup/nginx-$(date +%Y%m%d)` | `+%Y%m%d)` | 命令参数，用于指定要处理的对象或行为细节 |
| `cp -iv *.conf /etc/app/` | `-iv` | 命令参数，用于指定要处理的对象或行为细节 |
| `cp -iv *.conf /etc/app/` | `*.conf` | 输入或输出文件名，命令会对这个文件进行读取/写入/处理 |
| `cp -iv *.conf /etc/app/` | `/etc/app/` | 文件或目录路径，指定命令要操作的位置 |
| `cp -au /home/ace/projects/ /mnt/backup/projects/` | `-au` | 命令参数，用于指定要处理的对象或行为细节 |
| `cp -au /home/ace/projects/ /mnt/backup/projects/` | `/home/ace/projects/` | 文件或目录路径，指定命令要操作的位置 |
| `cp -au /home/ace/projects/ /mnt/backup/projects/` | `/mnt/backup/projects/` | 文件或目录路径，指定命令要操作的位置 |

---

### data/lessons/docker.toml

| Command | Token | Vague Pattern |
|---------|-------|---------------|
| `docker run -d --name webserver -p 8080:80 nginx` | `nginx` | 服务名/容器名，用来指定要查看或操作的目标 |
| `docker exec -it webserver bash` | `bash` | 命令参数，用于指定要处理的对象或行为细节 |
| `docker build -t myapp:v1.0 .` | `build` | 命令参数，用于指定要处理的对象或行为细节 |
| `docker build -t myapp:v1.0 .` | `-t` | 指定类型、压缩格式或算法类型 |
| `docker logs -f --tail 50 webserver` | `logs` | 命令参数，用于指定要处理的对象或行为细节 |
| `docker logs -f --tail 50 webserver` | `-f` | 持续跟随输出变化，或指定文件脚本（取决于命令） |
| `docker logs -f --tail 50 webserver` | `webserver` | 命令参数，用于指定要处理的对象或行为细节 |

---

### data/lessons/git.toml

| Command | Token | Vague Pattern |
|---------|-------|---------------|
| `git checkout -b feature/user-auth` | `feature/user-auth` | 文件或目录路径，指定命令要操作的位置 |
| `git add src/ && git commit -m '实现用户认证模块'` | `src/` | 文件或目录路径，指定命令要操作的位置 |
| `git stash && git checkout main && git pull` | `checkout` | 命令参数，用于指定要处理的对象或行为细节 |
| `git stash && git checkout main && git pull` | `pull` | 命令参数，用于指定要处理的对象或行为细节 |
| `git log --oneline --graph --all` | `log` | 命令参数，用于指定要处理的对象或行为细节 |

---

### data/lessons/head.toml

| Command | Token | Vague Pattern |
|---------|-------|---------------|
| `head -n 1 *.csv` | `*.csv` | 输入或输出文件名，命令会对这个文件进行读取/写入/处理 |
| `ls -lt \| head -6` | `-lt` | 命令参数，用于指定要处理的对象或行为细节 |
| `ls -lt \| head -6` | `-6` | 命令参数，用于指定要处理的对象或行为细节 |

---

### data/lessons/ip.toml

| Command | Token | Vague Pattern |
|---------|-------|---------------|
| `ip -br addr show` | `show` | 命令参数，用于指定要处理的对象或行为细节 |
| `ip route show` | `route` | 命令参数，用于指定要处理的对象或行为细节 |
| `ip route show` | `show` | 命令参数，用于指定要处理的对象或行为细节 |
| `ip link show` | `link` | 命令参数，用于指定要处理的对象或行为细节 |
| `ip link show` | `show` | 命令参数，用于指定要处理的对象或行为细节 |
| `ip neigh show` | `neigh` | 命令参数，用于指定要处理的对象或行为细节 |
| `ip neigh show` | `show` | 命令参数，用于指定要处理的对象或行为细节 |
| `sudo ip addr add 192.168.1.200/24 dev ens33` | `addr` | 命令参数，用于指定要处理的对象或行为细节 |
| `sudo ip addr add 192.168.1.200/24 dev ens33` | `add` | 命令参数，用于指定要处理的对象或行为细节 |
| `sudo ip addr add 192.168.1.200/24 dev ens33` | `192.168.1.200/24` | 文件或目录路径，指定命令要操作的位置 |
| `sudo ip addr add 192.168.1.200/24 dev ens33` | `dev` | 命令参数，用于指定要处理的对象或行为细节 |
| `sudo ip addr add 192.168.1.200/24 dev ens33` | `ens33` | 命令参数，用于指定要处理的对象或行为细节 |

---

### data/lessons/kill.toml

| Command | Token | Vague Pattern |
|---------|-------|---------------|
| `kill -HUP $(cat /var/run/nginx.pid)` | `-HUP` | 命令参数，用于指定要处理的对象或行为细节 |
| `kill -HUP $(cat /var/run/nginx.pid)` | `$(cat` | 命令参数，用于指定要处理的对象或行为细节 |
| `killall -v python3` | `python3` | 命令参数，用于指定要处理的对象或行为细节 |
| `pkill -f 'gunicorn.*myapp'` | `-f` | 持续跟随输出变化，或指定文件脚本（取决于命令） |

---

### data/lessons/mkdir.toml

| Command | Token | Vague Pattern |
|---------|-------|---------------|
| `mkdir projects` | `projects` | 命令参数，用于指定要处理的对象或行为细节 |
| `mkdir -pv /opt/myapp/logs /opt/myapp/config /opt/myapp/data` | `-pv` | 命令参数，用于指定要处理的对象或行为细节 |

---

### data/lessons/mv.toml

| Command | Token | Vague Pattern |
|---------|-------|---------------|
| `mv -v *.log /var/log/archive/` | `*.log` | 输入或输出文件名，命令会对这个文件进行读取/写入/处理 |
| `mv -i config.yml config.yml.bak` | `config.yml.bak` | 输入或输出文件名，命令会对这个文件进行读取/写入/处理 |
| `mv project-v1/ project-v2/` | `project-v2/` | 文件或目录路径，指定命令要操作的位置 |
| `find /tmp -name '*.tmp' -mtime +7 -exec mv {} /tmp/archive/ \;` | `/tmp` | 文件或目录路径，指定命令要操作的位置 |
| `find /tmp -name '*.tmp' -mtime +7 -exec mv {} /tmp/archive/ \;` | `*.tmp` | 输入或输出文件名，命令会对这个文件进行读取/写入/处理 |
| `find /tmp -name '*.tmp' -mtime +7 -exec mv {} /tmp/archive/ \;` | `/tmp/archive/` | 文件或目录路径，指定命令要操作的位置 |

---

### data/lessons/nginx.toml

| Command | Token | Vague Pattern |
|---------|-------|---------------|
| `ln -s ... && nginx -t && systemctl reload nginx` | `-t` | 指定类型、压缩格式或算法类型 |
| `ln -s ... && nginx -t && systemctl reload nginx` | `reload` | 命令参数，用于指定要处理的对象或行为细节 |
| `ln -s ... && nginx -t && systemctl reload nginx` | `nginx` | 服务名/容器名，用来指定要查看或操作的目标 |

---

### data/lessons/ping.toml

| Command | Token | Vague Pattern |
|---------|-------|---------------|
| `ping -c 4 8.8.8.8` | `8.8.8.8` | 命令参数，用于指定要处理的对象或行为细节 |
| `ping -c 3 google.com` | `google.com` | 命令参数，用于指定要处理的对象或行为细节 |
| `ping -c 4 192.168.1.1` | `192.168.1.1` | 命令参数，用于指定要处理的对象或行为细节 |
| `ping -qc 100 server.example.com` | `-qc` | 命令参数，用于指定要处理的对象或行为细节 |
| `ping -qc 100 server.example.com` | `server.example.com` | 命令参数，用于指定要处理的对象或行为细节 |
| `ping -c 4 -s 1400 10.0.0.1` | `-s` | 静默模式、包大小或排序方式（取决于命令） |
| `ping -c 4 -s 1400 10.0.0.1` | `1400` | 指定数据包大小或其他尺寸参数 |
| `ping -c 4 -s 1400 10.0.0.1` | `10.0.0.1` | 命令参数，用于指定要处理的对象或行为细节 |

---

### data/lessons/ps.toml

| Command | Token | Vague Pattern |
|---------|-------|---------------|
| `ps aux \| grep nginx` | `nginx` | 服务名/容器名，用来指定要查看或操作的目标 |
| `ps -eo pid,user,%cpu,%mem,comm --sort=-%cpu \| head -10` | `-10` | 命令参数，用于指定要处理的对象或行为细节 |
| `ps aux --sort=-%mem \| awk 'NR==1 \|\| $4>1.0'` | `aux` | 命令参数，用于指定要处理的对象或行为细节 |
| `ps aux --sort=-%mem \| awk 'NR==1 \|\| $4>1.0'` | `NR==1 \|\| $4>1.0` | 命令参数，用于指定要处理的对象或行为细节 |

---

### data/lessons/rm.toml

| Command | Token | Vague Pattern |
|---------|-------|---------------|
| `rm -v *.tmp *.bak` | `*.tmp` | 输入或输出文件名，命令会对这个文件进行读取/写入/处理 |
| `rm -v *.tmp *.bak` | `*.bak` | 输入或输出文件名，命令会对这个文件进行读取/写入/处理 |
| `find /tmp -type f -mtime +30 -delete` | `/tmp` | 文件或目录路径，指定命令要操作的位置 |
| `find . -name '*.pyc' -type f -exec rm -v {} +` | `f` | 命令参数，用于指定要处理的对象或行为细节 |

---

### data/lessons/sed.toml

| Command | Token | Vague Pattern |
|---------|-------|---------------|
| `sed '/^#/d; /^$/d' /etc/ssh/sshd_config` | `/^#/d; /^$/d` | 这是文本处理表达式，用来定义匹配规则和输出/替换动作 |
| `sed -i.bak 's/localhost/db.production.internal/g' config.yml` | `s/localhost/db.production.internal/g` | 这是文本处理表达式，用来定义匹配规则和输出/替换动作 |
| `sed -E 's/([0-9]{4})-([0-9]{2})-([0-9]{2})/\3\/\2\/\1/g' dates.txt` | `s/([0-9]{4})-([0-9]{2})-([0-9]{2})/\3\/\2\/\1/g` | 这是文本处理表达式，用来定义匹配规则和输出/替换动作 |

---

### data/lessons/sort.toml

| Command | Token | Vague Pattern |
|---------|-------|---------------|
| `sort -t: -k3 -n /etc/passwd \| head -5` | `-k3` | 命令参数，用于指定要处理的对象或行为细节 |
| `sort -t: -k3 -n /etc/passwd \| head -5` | `-5` | 命令参数，用于指定要处理的对象或行为细节 |
| `du -sh /var/log/* \| sort -rh \| head -10` | `-sh` | 命令参数，用于指定要处理的对象或行为细节 |
| `du -sh /var/log/* \| sort -rh \| head -10` | `-rh` | 命令参数，用于指定要处理的对象或行为细节 |
| `du -sh /var/log/* \| sort -rh \| head -10` | `-10` | 命令参数，用于指定要处理的对象或行为细节 |
| `cat access.log \| awk '{print $1}' \| sort \| uniq -c \| sort -rn \| head -10` | `{print $1}` | 命令参数，用于指定要处理的对象或行为细节 |
| `cat access.log \| awk '{print $1}' \| sort \| uniq -c \| sort -rn \| head -10` | `-rn` | 命令参数，用于指定要处理的对象或行为细节 |
| `cat access.log \| awk '{print $1}' \| sort \| uniq -c \| sort -rn \| head -10` | `-10` | 命令参数，用于指定要处理的对象或行为细节 |

---

### data/lessons/ssh.toml

| Command | Token | Vague Pattern |
|---------|-------|---------------|
| `ssh user@server uptime` | `uptime` | 命令参数，用于指定要处理的对象或行为细节 |
| `ssh-keygen -t ed25519 -C "user@workstation"` | `-t` | 指定类型、压缩格式或算法类型 |
| `ssh-keygen -t ed25519 -C "user@workstation"` | `-C` | 添加注释或指定目录/上下文（具体取决于命令） |
| `ssh-keygen -t ed25519 -C "user@workstation"` | `user@workstation` | 这里通常是注释文本或目标目录参数 |
| `ssh -R 8080:localhost:3000 user@public-server` | `-R` | 递归处理目录，或建立反向端口转发 |
| `ssh -R 8080:localhost:3000 user@public-server` | `8080:localhost:3000` | 反向转发规则：远端端口:本地主机:本地端口 |

---

### data/lessons/systemctl.toml

| Command | Token | Vague Pattern |
|---------|-------|---------------|
| `systemctl enable --now docker` | `docker` | 服务名/容器名，用来指定要查看或操作的目标 |
| `journalctl -u nginx -f --since '10 minutes ago'` | `--since` | 命令参数，用于指定要处理的对象或行为细节 |

---

### data/lessons/tail.toml

| Command | Token | Vague Pattern |
|---------|-------|---------------|
| `tail -f /var/log/syslog \| grep --line-buffered ERROR` | `/var/log/syslog` | 文件或目录路径，指定命令要操作的位置 |
| `tail -f /var/log/syslog \| grep --line-buffered ERROR` | `ERROR` | 命令参数，用于指定要处理的对象或行为细节 |

---

### data/lessons/tar.toml

| Command | Token | Vague Pattern |
|---------|-------|---------------|
| `tar -czvf backup.tar.gz --exclude='*.log' --exclude='node_modules' project/` | `--exclude=node_modules` | 命令参数，用于指定要处理的对象或行为细节 |
| `tar -czvf backup.tar.gz --exclude='*.log' --exclude='node_modules' project/` | `project/` | 文件或目录路径，指定命令要操作的位置 |

---

### data/lessons/top.toml

| Command | Token | Vague Pattern |
|---------|-------|---------------|
| `top -bn1 \| head -20` | `-bn1` | 命令参数，用于指定要处理的对象或行为细节 |
| `top -bn1 \| head -20` | `-20` | 命令参数，用于指定要处理的对象或行为细节 |
| `top -d 1 -p $(pgrep -d, nginx)` | `-d,` | 命令参数，用于指定要处理的对象或行为细节 |
| `top -d 1 -p $(pgrep -d, nginx)` | `nginx)` | 命令参数，用于指定要处理的对象或行为细节 |
| `top -bn1 -o %MEM \| head -17 \| tail -10` | `-bn1` | 命令参数，用于指定要处理的对象或行为细节 |
| `top -bn1 -o %MEM \| head -17 \| tail -10` | `-17` | 命令参数，用于指定要处理的对象或行为细节 |
| `top -bn1 -o %MEM \| head -17 \| tail -10` | `-10` | 命令参数，用于指定要处理的对象或行为细节 |

---

### data/lessons/uniq.toml

| Command | Token | Vague Pattern |
|---------|-------|---------------|
| `sort access.log \| uniq -c \| sort -rn \| head -5` | `-5` | 命令参数，用于指定要处理的对象或行为细节 |
| `awk '{print $1}' access.log \| sort \| uniq -c \| sort -rn \| head -10` | `{print $1}` | 命令参数，用于指定要处理的对象或行为细节 |
| `awk '{print $1}' access.log \| sort \| uniq -c \| sort -rn \| head -10` | `-rn` | 命令参数，用于指定要处理的对象或行为细节 |
| `awk '{print $1}' access.log \| sort \| uniq -c \| sort -rn \| head -10` | `-10` | 命令参数，用于指定要处理的对象或行为细节 |

---

### data/lessons/wc.toml

| Command | Token | Vague Pattern |
|---------|-------|---------------|
| `wc -l src/*.rs` | `src/*.rs` | 输入或输出文件名，命令会对这个文件进行读取/写入/处理 |
| `cat access.log \| grep '404' \| wc -l` | `access.log` | 输入或输出文件名，命令会对这个文件进行读取/写入/处理 |
| `find . -name '*.rs' -exec cat {} + \| wc -l` | `*.rs` | 输入或输出文件名，命令会对这个文件进行读取/写入/处理 |

---

### data/lessons/xargs.toml

| Command | Token | Vague Pattern |
|---------|-------|---------------|
| `find . -name '*.log' -print0 \| xargs -0 wc -l` | `wc` | 命令参数，用于指定要处理的对象或行为细节 |
| `echo 1 2 3 4 5 6 \| xargs -n 2 echo` | `echo` | 命令参数，用于指定要处理的对象或行为细节 |
| `ls *.jpg \| xargs -I {} convert {} -resize 50% thumb_{}` | `*.jpg` | 输入或输出文件名，命令会对这个文件进行读取/写入/处理 |
| `ls *.jpg \| xargs -I {} convert {} -resize 50% thumb_{}` | `convert` | 命令参数，用于指定要处理的对象或行为细节 |
| `ls *.jpg \| xargs -I {} convert {} -resize 50% thumb_{}` | `-resize` | 命令参数，用于指定要处理的对象或行为细节 |
| `ls *.jpg \| xargs -I {} convert {} -resize 50% thumb_{}` | `50%` | 命令参数，用于指定要处理的对象或行为细节 |
| `ls *.jpg \| xargs -I {} convert {} -resize 50% thumb_{}` | `thumb_{}` | 命令参数，用于指定要处理的对象或行为细节 |
| `cat urls.txt \| xargs -n 1 -P 4 wget -q` | `urls.txt` | 输入或输出文件名，命令会对这个文件进行读取/写入/处理 |
| `cat urls.txt \| xargs -n 1 -P 4 wget -q` | `wget` | 命令参数，用于指定要处理的对象或行为细节 |
| `grep -rl 'old_api' ./src \| xargs sed -i 's/old_api/new_api/g'` | `-rl` | 命令参数，用于指定要处理的对象或行为细节 |
| `grep -rl 'old_api' ./src \| xargs sed -i 's/old_api/new_api/g'` | `old_api` | 命令参数，用于指定要处理的对象或行为细节 |
| `grep -rl 'old_api' ./src \| xargs sed -i 's/old_api/new_api/g'` | `sed` | 命令参数，用于指定要处理的对象或行为细节 |

---

## 3. Command Files (`data/commands/*.toml`)

✅ **No vague descriptions found in any command files (01–19).** All 19 command TOML files are clean.

---

## 4. TOML Parse Errors

✅ **No parse errors.** All 50 TOML files (31 lesson + 19 command) parsed successfully with Python `tomllib`.

---

## 5. Data Corruption / Misplaced Content

No evidence of broken or cross-contaminated token_details entries was found. All vague entries are consistently using the template strings listed above — they are clearly placeholder/template text that was never replaced with specific descriptions.

**Notable quality issues (beyond raw vague-pattern matching):**

- **ip.toml**: IP address `192.168.1.200/24` described as "文件或目录路径" — factually incorrect, this is a network address/prefix, not a file path.
- **ping.toml**: Hostnames and IP addresses (e.g., `8.8.8.8`, `google.com`) described as generic "命令参数" — should specifically say "要 ping 的目标主机/IP 地址".
- **kill.toml**: `pkill -f` token `-f` described as "持续跟随输出变化" — this is the wrong vague pattern; `-f` in `pkill` means "match against full command line", not "follow".
- **docker.toml**: `-t` in `docker build -t myapp:v1.0 .` described as "指定类型、压缩格式或算法类型" — wrong; `-t` here means "tag the image with a name".
- **nginx.toml**: `nginx -t` flag described as "指定类型、压缩格式或算法类型" — wrong; `-t` here tests the nginx configuration syntax.
- **ssh.toml**: `-R` described as "递归处理目录，或建立反向端口转发" — the "递归处理目录" part is from `cp -r`/`rm -r`, completely wrong context for SSH.
- **git.toml**: `feature/user-auth` described as "文件或目录路径" — this is a branch name, not a file path.
- **mkdir.toml**: `projects` described as "命令参数，用于指定要处理的对象或行为细节" — should be "要创建的目录名称".

---

## 6. Verdict

**FAIL**

- **113 vague descriptions** remain across **24 lesson files**
- **0 vague descriptions** in command files (PASS for that section)
- **0 TOML parse errors**
- **0 data corruption issues**

The lesson files still contain widespread use of template placeholder text. Every flagged entry needs a specific, command-context-aware explanation.

---

*Audit completed: 2026-03-21 by Professor Socrates*
