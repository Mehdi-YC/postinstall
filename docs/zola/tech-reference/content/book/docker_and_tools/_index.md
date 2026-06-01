+++
title = "General Dev Reference"
description = "Git, Docker, Nginx, Linux, and essential developer tools — commands, patterns, and gotchas"
[extra]
date = 2024-01-15
toc = true
+++

## Git

### Setup & Config
```bash
    git config --global user.name "Your Name"
    git config --global user.email "you@example.com"
    git config --global init.defaultBranch main
    git config --global core.autocrlf input   # Windows: fix line endings
    git config --global pull.rebase false      # merge on pull (default)
    git config --global --list                  # show all config
```
### Basic Workflow
```bash
    git init                                    # init repo in current dir
    git clone https://github.com/user/repo.git   # clone remote
    git clone --depth 1 https://...             # shallow clone (no history)
    git status                                  # see changes
    git add .                                    # stage everything
    git add file.py                              # stage one file
    git add -p                                   # stage interactively (per hunk)
    git commit -m "add user authentication"
    git push origin main                         # push to remote
    git pull origin main                         # pull + merge
    git fetch origin                             # download without merging
```
### Branching
```bash
    git branch                                  # list local branches
    git branch -a                               # list all (including remote)
    git checkout -b feature/login               # create + switch
    git switch -c feature/login                  # same (newer syntax)
    git switch main                             # switch back
    git merge feature/login                      # merge into current branch
    git branch -d feature/login                 # delete merged branch
    git branch -D feature/login                 # delete even if not merged
    git push origin --delete feature/login       # delete remote branch
```
### Undoing Things

| Scenario | Command |
|----------|---------|
| Unstage a file | `git restore --staged file.py` |
| Discard working dir changes | `git restore file.py` |
| Amend last commit (no push yet) | `git commit --amend -m "new msg"` |
| Undo last commit, keep changes | `git reset --soft HEAD~1` |
| Undo last commit, discard changes | `git reset --hard HEAD~1` |
| Undo last commit (safe, if pushed) | `git revert HEAD` (creates new commit) |
| Unstage everything | `git restore --staged .` |
| See what changed in a commit | `git show HEAD` or `git show abc123` |

> **`reset --hard` is destructive.** It discards working changes. Use `--soft` if you want to keep changes staged, or `revert` if the commit was already pushed.

### Rebase vs Merge

| | Merge | Rebase |
|--|-------|--------|
| History | Preserves all branches (messy) | Linear history (clean) |
| Safety | Safe, doesn't rewrite history | Rewrites commit hashes |
| When to use | Shared branches, merges to main | Local feature branches before merging |
| Rule | Default for shared work | Never rebase commits that others have pulled |
```bash
    # Interactive rebase: edit/squash/reorder last 3 commits
    git rebase -i HEAD~3

    # Rebase feature branch onto latest main (before merging)
    git checkout feature/login
    git rebase main

    # Abort a rebase if it goes wrong
    git rebase --abort
```
### Stash
```bash
    git stash                                   # save current changes
    git stash save "work in progress"        # with message
    git stash list                              # list stashes
    git stash pop                               # restore + remove from list
    git stash apply                              # restore + keep in list
    git stash drop                              # remove specific stash
```
### Log & Diff
```bash
    git log --oneline --graph --all             # visual branch history
    git log --oneline -10                        # last 10 commits
    git log --author="name"                     # filter by author
    git log --since="2 weeks ago"               # filter by date
    git log -p file.py                          # log + diffs for one file
    git diff                                    # unstaged changes
    git diff --staged                           # staged changes
    git diff main..feature                      # diff between branches
    git blame file.py                           # who changed each line
    git shortlog -sn                            # commit count per author
```
### .gitignore Essentials

    # Files
    .env
    *.log
    *.pyc
    __pycache__/
    node_modules/
    .DS_Store

    # Directories
    db.sqlite3
    media/
    staticfiles/
    *.egg-info/
    dist/
    build/

    # Patterns
    *.swp
    *.swo
    *~
    .idea/
    .vscode/

### Fixing Mistakes
```bash
    # Remove a file from git tracking (keep on disk)
    git rm --cached file.py

    # Remove a file from git AND disk
    git rm file.py

    # Undo a pushed commit (creates a new commit that undoes it)
    git revert abc123

    # Change last commit message (before push)
    git commit --amend -m "corrected message"

    # Recover a deleted branch (find hash in reflog first)
    git reflog
    git checkout -b recovered-branch abc123

    # Clean untracked files and directories (dry run first)
    git clean -n -d
    git clean -f -d
```
## Docker

### Basic Commands

| Command | What It Does |
|---------|---------------|
| `docker run nginx` | Pull + run an image |
| `docker run -d nginx` | Run in background (detached) |
| `docker run -it ubuntu bash` | Interactive shell inside container |
| `docker run -p 8080:80 nginx` | Map host 8080 to container 80 |
| `docker run --rm nginx` | Auto-remove container when it stops |
| `docker run --name mynginx nginx` | Name the container |
| `docker run -e VAR=val nginx` | Set environment variable |
| `docker run -v /host:/container nginx` | Bind mount a volume |
| `docker run -v myvol:/data nginx` | Mount a named volume |
| `docker run --network mynet nginx` | Connect to a network |
| `docker build -t myapp .` | Build image from Dockerfile |
| `docker ps` | List running containers |
| `docker ps -a` | List all containers (including stopped) |
| `docker stop <id>` | Graceful stop |
| `docker kill <id>` | Force kill |
| `docker start <id>` | Start a stopped container |
| `docker restart <id>` | Restart |
| `docker rm <id>` | Remove stopped container |
| `docker rm -f <id>` | Force remove (even if running) |
| `docker logs <id>` | View container logs |
| `docker logs -f <id>` | Follow logs (like tail -f) |
| `docker exec -it <id> bash` | Shell into running container |
| `docker cp <id>:/file ./local` | Copy file from container |
| `docker images` | List local images |
| `docker rmi <image>` | Remove an image |
| `docker system prune` | Remove all unused containers, images, networks |
| `docker system prune -a --volumes` | Nuclear: remove everything unused |

### Dockerfile Example (Python app)
```dockerfile
    # ---- Build stage ----
    FROM python:3.12-slim AS builder

    WORKDIR /build
    # Install build dependencies only (not in final image)
    RUN apt-get update && apt-get install -y --no-install-recommends \
        gcc libpq-dev && \
        rm -rf /var/lib/apt/lists/*

    COPY requirements.txt .
    RUN pip install --no-cache-dir --user -r requirements.txt

    # ---- Runtime stage ----
    FROM python:3.12-slim

    WORKDIR /app

    # Copy only installed packages from builder (no build tools)
    COPY --from=builder /root/.local /root/.local
    ENV PATH=/root/.local/bin:$PATH

    # Install only runtime dependencies
    RUN apt-get update && apt-get install -y --no-install-recommends \
        libpq5 curl && \
        rm -rf /var/lib/apt/lists/*

    COPY . .

    # Non-root user for security
    RUN useradd --create-home appuser
    USER appuser

    EXPOSE 8000

    CMD ["gunicorn", "myproject.wsgi:application", "--bind", "0.0.0.0:8000"]
```
### Dockerfile Example (Node.js app)
```dockerfile
    FROM node:20-alpine AS builder
    WORKDIR /app
    COPY package*.json ./
    RUN npm ci
    COPY . .
    RUN npm run build

    FROM node:20-alpine
    WORKDIR /app
    COPY --from=builder /app/dist ./dist
    COPY --from=builder /app/node_modules ./node_modules
    COPY --from=builder /app/package.json ./
    EXPOSE 3000
    CMD ["node", "server.js"]
```
> **Multi-stage builds reduce image size dramatically.** The builder stage has compilers and build tools; the runtime stage only has what's needed to run. A Python build stage might be 800MB; the runtime stage can be 150MB.

### Useful Flags

| Flag | Purpose |
|------|---------|
| `-d` | Detached mode (background) |
| `-it` | Interactive + TTY (for shell access) |
| `--rm` | Remove container after it exits |
| `-p 8080:80` | Port mapping (host:container) |
| `-P` | Map all exposed ports to random host ports |
| `-v /host:/container` | Bind mount (two-way, not copied) |
| `-e KEY=VAL` | Environment variable |
| `--env-file .env` | Load env vars from file |
| `--name` | Name the container |
| `--network` | Connect to a specific network |
| `--restart unless-stopped` | Auto-restart (except after manual stop) |
| `--read-only` | Read-only filesystem (security) |
| `--memory 512m` | Limit memory |
| `--cpus 1.5` | Limit CPU |
| `-w /app` | Set working directory |
| `--no-cache` | Build without cache (docker build) |

### Volumes
```bash
    # Named volume (managed by Docker, stored in /var/lib/docker/volumes/)
    docker volume create mydata
    docker run -v mydata:/var/lib/postgresql/data postgres

    # Bind mount (maps a host path directly)
    docker run -v $(pwd)/src:/app/src myapp

    # Anonymous volume (removed with container)
    docker run -v /app/data myapp

    # List volumes
    docker volume ls

    # Remove unused volumes
    docker volume prune
```
> **Named volumes for persistent data, bind mounts for development.** Named volumes are portable across hosts. Bind mounts are faster for dev because edits on the host are immediately visible in the container.

### Networks
```bash
    docker network create mynet
    docker run --network mynet --name app myapp
    docker run --network mynet --name db postgres
    # app can now reach db at hostname "db" (the container name)

    # Default bridge: containers can reach each other by IP, NOT by name.
    # Custom bridge (like mynet): DNS resolution by container name works.
    # host network: container shares host's network stack (no port mapping needed).
    # none: no network access.
```
## Docker Compose

### Commands

| Command | What It Does |
|---------|---------------|
| `docker compose up -d` | Build + start all services (detached) |
| `docker compose up --build` | Rebuild images before starting |
| `docker compose down` | Stop + remove containers + networks |
| `docker compose down -v` | Also remove volumes (destroys data) |
| `docker compose logs -f app` | Follow logs for one service |
| `docker compose ps` | Status of all services |
| `docker compose exec app bash` | Shell into a service container |
| `docker compose run --rm app python manage.py migrate` | Run one-off command in a service |
| `docker compose build` | Build all images |
| `docker compose build app` | Build one service |
| `docker compose pull` | Pull latest images |
| `docker compose restart app` | Restart one service |
| `docker compose stop` | Stop without removing |
| `docker compose start` | Start stopped services |
| `docker compose config` | Validate + view merged config |
| `docker compose top` | Show running processes in services |

### docker-compose.yml (Django + PostgreSQL)
```yaml
    services:
      db:
        image: postgres:16-alpine
        restart: unless-stopped
        volumes:
          - postgres_data:/var/lib/postgresql/data
        environment:
          POSTGRES_DB: mydb
          POSTGRES_USER: myuser
          POSTGRES_PASSWORD: mypassword
        ports:
          - "5432:5432"
        healthcheck:
          test: ["CMD-SHELL", "pg_isready -U myuser -d mydb"]
          interval: 5s
          timeout: 5s
          retries: 5

      web:
        build: .
        restart: unless-stopped
        volumes:
          - .:/app                          # bind mount source code
          - static_files:/app/staticfiles   # named volume for collectstatic
          - media_files:/app/media
        ports:
          - "8000:8000"
        environment:
          - DEBUG=False
          - SECRET_KEY=changeme-in-production
          - DB_HOST=db
          - DB_NAME=mydb
          - DB_USER=myuser
          - DB_PASSWORD=mypassword
        depends_on:
          db:
            condition: service_healthy    # wait until db is ready

      nginx:
        image: nginx:alpine
        restart: unless-stopped
        ports:
          - "80:80"
          - "443:443"
        volumes:
          - ./nginx.conf:/etc/nginx/conf.d/default.conf:ro
          - static_files:/usr/share/nginx/static:ro
          - media_files:/usr/share/nginx/media:ro
          - ./certs:/etc/nginx/certs:ro
        depends_on:
          - web

    volumes:
      postgres_data:
      static_files:
      media_files:
```
> **`depends_on` with `condition: service_healthy`** is the right pattern. Without it, web might start before the database is ready to accept connections. Always add healthchecks to databases.

### Override Files (dev only)
```yaml
    # docker-compose.override.yml - This file is automatically loaded by docker compose.
    # Add it to .gitignore so dev overrides don't reach production.
    services:
      web:
        command: python manage.py runserver 0.0.0.0:8000
        environment:
          - DEBUG=True
        volumes:
          - .:/app
        # No restart policy, no port restriction in dev
```
> **`docker-compose.override.yml` is auto-loaded.** Use it for dev-only settings (debug mode, different command, extra volumes). Keep `docker-compose.yml` production-ready. Add `docker-compose.override.yml` to `.gitignore`.

### Common Patterns
```bash
    # Run a one-off command (migrate, createsuperuser, etc.)
    docker compose run --rm web python manage.py migrate
    docker compose run --rm web python manage.py createsuperuser

    # Run tests
    docker compose run --rm web python manage.py test

    # Rebuild only one service
    docker compose build web
    docker compose up -d web

    # View resource usage
    docker stats

    # Use multiple compose files
    docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d
```
## Nginx

### Config File Location

    # Main config:
    /etc/nginx/nginx.conf

    # Server blocks (virtual hosts):
    /etc/nginx/conf.d/default.conf      # Debian/Ubuntu
    /etc/nginx/sites-available/        # Debian (with sites-enabled symlink pattern)
    /etc/nginx/conf.d/*.conf           # All .conf files here are auto-loaded

### Reverse Proxy (Python/Django)
```conf
    # /etc/nginx/conf.d/myapp.conf
    upstream django_app {
        server 127.0.0.1:8000;
    }

    server {
        listen 80;
        server_name example.com www.example.com;

        # Redirect HTTP to HTTPS
        return 301 https://$host$request_uri;

        # Or serve directly on HTTP (no SSL):
        # location / {
        #     proxy_pass http://django_app;
        # }
    }

    server {
        listen 443 ssl http2;
        server_name example.com www.example.com;

        ssl_certificate     /etc/letsencrypt/live/example.com/fullchain.pem;
        ssl_certificate_key /etc/letsencrypt/live/example.com/privkey.pem;
        ssl_protocols       TLSv1.2 TLSv1.3;
        ssl_ciphers         HIGH:!aNULL:!MD5;

        # Security headers
        add_header X-Frame-Options DENY;
        add_header X-Content-Type-Options nosniff;
        add_header X-XSS-Protection "1; mode=block";
        add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;

        # Max upload size (default: 1MB, often too small)
        client_max_body_size 50M;

        # Static files (served directly by nginx, no app needed)
        location /static/ {
            alias /usr/share/nginx/static/;
            expires 30d;
            add_header Cache-Control "public, immutable";
        }

        # Media files (user uploads)
        location /media/ {
            alias /usr/share/nginx/media/;
            expires 7d;
        }

        # Proxy to app
        location / {
            proxy_pass http://django_app;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
        }

        # Block common attack paths
        location ~ /\.(git|env|htpasswd|htaccess) {
            deny all;
            return 404;
        }
    }
```
### Reverse Proxy (Node.js)
```conf
    # /etc/nginx/conf.d/nodeapp.conf
    upstream node_app {
        server 127.0.0.1:3000;
    }

    server {
        listen 80;
        server_name app.example.com;

        location / {
            proxy_pass http://node_app;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;

            # WebSocket support (for hot reload / Socket.io)
            proxy_http_version 1.1;
            proxy_set_header Upgrade $http_upgrade;
            proxy_set_header Connection "upgrade";
        }
    }
```
### Useful Directives

| Directive | Purpose |
|-----------|---------|
| `listen 80;` | Port to listen on |
| `listen 443 ssl http2;` | HTTPS + HTTP/2 |
| `server_name` | Domain names this block matches |
| `root` | Document root directory |
| `index` | Default files to serve (index.html) |
| `location /path/` | URL matching (prefix, regex, exact) |
| `location = /exact` | Exact match (no sub-paths) |
| `location ~ \.php$` | Case-sensitive regex match |
| `location ~* \.(jpg|png)$` | Case-insensitive regex |
| `proxy_pass` | Forward requests to upstream |
| `rewrite` | URL rewriting |
| `return` | Return status code + redirect |
| `try_files` | Fallback chain: try file, try dir, return |
| `client_max_body_size` | Max upload size (default 1MB) |
| `expires` | Cache expiration for responses |
| `add_header` | Add response header |
| `gzip on;` | Enable gzip compression |
| `access_log` / `error_log` | Log file paths |
| `deny / allow` | IP-based access control |

### Common Operations
```bash
    nginx -t                # test config syntax (always run before reload)
    nginx -s reload        # reload config without dropping connections
    nginx -s stop           # stop
    nginx -s reopen        # reopen log files (for log rotation)
```
> **Always run `nginx -t` before `nginx -s reload`.** A syntax error in any conf file will prevent the reload and leave nginx in its last working state, but it's still good practice.

### SPA Fallback (Vue/React/Angular)

    location / {
        root /usr/share/nginx/html;
        try_files $uri $uri/ /index.html;   # serve index.html for all routes
    }

### Gzip Compression

    gzip on;
    gzip_types text/plain text/css application/json application/javascript
               text/xml application/xml application/xml+rss text/javascript
               image/svg+xml;
    gzip_min_length 256;
    gzip_vary on;

## Linux

### Navigation & Files

    pwd                    # print working directory
    cd /path/to/dir        # change directory
    cd ..                   # parent directory
    cd -                    # previous directory
    ls -la                  # list all (including hidden) with details
    ls -lh                  # human-readable file sizes
    tree -L 2               # directory tree (2 levels)
    find . -name "*.py"      # find files by name
    find . -type f -mtime -7  # files modified in last 7 days
    find . -size +100M       # files larger than 100MB
    find . -name "__pycache__" -type d -exec rm -rf {} +  # find + delete

### File Operations
```bash
    cp file.py backup.py   # copy
    cp -r dir/ dir_copy/   # copy directory recursively
    mv old.py new.py       # move / rename
    rm file.py             # delete
    rm -r dir/              # delete directory recursively
    rm -rf dir/             # force delete (no prompts)
    mkdir -p a/b/c          # create nested directories
    ln -s target link       # create symbolic link
    touch file.py           # create empty file / update timestamp
    du -sh dir/             # directory size (human-readable, summary)
    du -sh * | sort -rh     # largest items in current dir
    df -h                   # disk space usage per filesystem
    df -h /                  # disk space for root filesystem
```
### Viewing Files
```bash
    cat file.py             # print entire file
    less file.py            # pager (q to quit, / to search)
    head -20 file.py        # first 20 lines
    tail -20 file.py        # last 20 lines
    tail -f log.txt        # follow file (like tail -f)
    wc -l file.py          # line count
    file file.py            # file type detection
```
### Search & Text Processing
```bash
    grep "pattern" file.py              # search in file
    grep -r "pattern" .                # recursive search
    grep -ri "pattern" --include="*.py" .  # recursive, case-insensitive, specific files
    grep -n "pattern" file.py          # show line numbers
    grep -C 3 "pattern" file.py        # 3 lines of context around match
    grep -v "pattern" file.py          # lines NOT matching
    grep -c "pattern" file.py          # count matching lines

    sed -i 's/old/new/g' file.py       # find and replace in file
    sed -i 's/old/new/g' *.py          # find and replace in multiple files

    sort file.txt                     # sort lines
    sort -u file.txt                  # sort + remove duplicates
    sort -n file.txt                  # numeric sort
    sort -k2 -n file.txt             # sort by 2nd field numerically

    uniq file.txt                     # remove adjacent duplicates (pipe after sort)
    sort file.txt | uniq -c            # count duplicates

    awk '{print $1}' file.txt          # print first column
    awk '{sum+=$1} END {print sum}'   # sum first column

    cut -d: -f1 file.txt             # extract first field (delimiter :)
    cut -d, -f1,3 file.txt           # extract fields 1 and 3 (delimiter ,)
```
### Permissions
```bash
    # r=4, w=2, x=1. Owner | Group | Others
    chmod 755 script.sh       # rwxr-xr-x (owner: full, others: read+exec)
    chmod 644 file.txt        # rw-r--r-- (standard file)
    chmod +x script.sh        # make executable
    chmod -R 755 dir/         # recursive
    chown user:group file    # change owner and group
    chown -R user:group dir/ # recursive
```
### Processes
```bash
    ps aux                          # all processes
    ps aux | grep python            # find python processes
    ps -p 1234 -o pid,cmd          # info about specific PID
    top                             # interactive process viewer
    htop                            # better top (install separately)
    kill 1234                       # send SIGTERM (graceful stop)
    kill -9 1234                    # send SIGKILL (force kill)
    killall python                  # kill all processes by name
    lsof -i :8000                   # what's using port 8000
    ss -tlnp                        # listening ports (modern, faster than netstat)
    nohup python app.py &            # run in background, survives logout
    disown                          # detach from current shell
```
### Systemd (Service Management)
```bash
    sudo systemctl start myapp.service      # start
    sudo systemctl stop myapp.service       # stop
    sudo systemctl restart myapp.service    # restart
    sudo systemctl enable myapp.service     # start on boot
    sudo systemctl disable myapp.service    # don't start on boot
    sudo systemctl status myapp.service     # check status
    sudo systemctl reload nginx.service     # reload config (no downtime)
    journalctl -u myapp -f                 # follow logs for a service
    journalctl -u myapp --since "1 hour ago"  # logs since time
```
### Archives
```bash
    tar -czf archive.tar.gz dir/       # create compressed archive
    tar -xzf archive.tar.gz           # extract
    tar -xzf archive.tar.gz -C /dest  # extract to specific directory
    tar -tzf archive.tar.gz           # list contents
    zip -r archive.zip dir/            # create zip
    unzip archive.zip                   # extract zip
```
### Networking
```bash
    curl -I https://example.com           # fetch headers only
    curl -s https://api.example.com/data | jq .  # fetch + parse JSON
    curl -X POST -H "Content-Type: application/json" -d '{"key":"val"}' url
    curl -o file.zip https://example.com/file.zip  # download file
    wget https://example.com/file.zip   # download (simpler than curl for files)
    ping example.com                    # check if host is reachable
    dig example.com                     # DNS lookup
    nslookup example.com                # DNS lookup (simpler)
    ssh user@server.com                # SSH into server
    scp file.py user@server:~/         # copy file to server
    scp user@server:~/file.py .       # copy file from server
```
### Pipes & Redirection
```bash
    command > file.txt           # redirect stdout to file (overwrite)
    command >> file.txt          # redirect stdout to file (append)
    command 2> error.log         # redirect stderr to file
    command > /dev/null 2>&1     # discard all output
    command1 | command2        # pipe stdout of cmd1 to stdin of cmd2
    command | tee output.txt    # output to stdout AND file
    command | xargs -I {} cmd  # use output as arguments (replaces {})
    cat files.txt | xargs rm   # delete files listed in a file
```
### Disk & Memory
```bash
    free -h                   # memory usage
    df -h                     # disk usage per filesystem
    du -sh /path/to/dir       # directory size
    du -sh * | sort -rh | head  # top 10 largest in current dir
    ncdu /path               # interactive disk usage explorer (install separately)
```
### Cron (Scheduled Tasks)
```bash
    crontab -e                  # edit your crontab
    crontab -l                  # list your crontab

    # Format: minute hour day-of-month month day-of-week command
    # */5 * * * * = every 5 minutes
    # 0 2 * * *   = every day at 2 AM
    # 0 */6 * * * = every 6 hours
    # 0 0 * * 1   = every Monday at midnight
    # 30 4 * * 1-5 = Monday-Friday at 4:30 AM

    # Examples:
    */5 * * * * /usr/bin/python3 /app/manage.py runcrons >> /var/log/cron.log 2>&1
    0 3 * * * /usr/bin/docker system prune -f >> /var/log/docker-prune.log 2>&1
```
### SSH Config
```conf
    # ~/.ssh/config
    Host myserver
        HostName 192.168.1.100
        User deploy
        Port 22
        IdentityFile ~/.ssh/mykey
        ForwardAgent yes

    Host github
        HostName github.com
        User git
        IdentityFile ~/.ssh/github_key
```
## Other Essential Tools

### tmux (Terminal Multiplexer)
```bash
    tmux                        # start new session
    tmux new -s mysession     # named session
    tmux ls                      # list sessions
    tmux attach -t mysession  # reattach (survives disconnect)
    tmux kill-session -t s     # kill session
```
```
    # Inside tmux (prefix = Ctrl+b by default):
    Ctrl+b c                       # new window
    Ctrl+b %                       # split pane horizontally
    Ctrl+b "                       # split pane vertically
    Ctrl+b n / Ctrl+b p            # next / previous pane
    Ctrl+b arrow-keys             # navigate between panes
    Ctrl+b d                       # detach (session keeps running)
    Ctrl+b [                       # scroll mode (q to quit, arrows to scroll)
    Ctrl+b ,                       # rename window
```
> **tmux is essential for remote servers.** Start a long-running task, detach, close your laptop, reconnect later. The process keeps running. No more lost SSH sessions.

### jq (JSON Processor)
```bash
    echo '{"name": "Alice", "age": 30}' | jq .
    echo '{"name": "Alice"}' | jq .name
    echo '[1,2,3]' | jq '.[]'
    echo '{"items": [{"id":1},{"id":2}]}' | jq '.items[].id'
    curl -s https://api.example.com/data | jq '.results | length'
    curl -s https://api.example.com/data | jq 'select(.active)'
    cat file.json | jq 'sort_by(.created_at) | reverse | .[0:10]'
```
### curl (HTTP Client)
```bash
    curl url                       # GET request
    curl -I url                     # headers only
    curl -s url                     # silent (no progress bar)
    curl -o file url               # download to file
    curl -X POST url -d "key=val"  # POST form data
    curl -X POST url -H "Content-Type: application/json" -d '{"key":"val"}'
    curl -H "Authorization: Bearer TOKEN" url  # with auth header
    curl -v url                     # verbose (see full request/response)
    curl -w "\n%{http_code}\n" url  # print only status code
    curl -w "\n%{time_total}\n" url  # measure response time
```
### xargs (Argument Builder)
```bash
    # Run a command on every line of input
    echo -e "file1\nfile2\nfile3" | xargs rm

    # Run with placeholder
    find . -name "*.log" | xargs -I {} mv {} /logs/

    # Parallel execution
    find . -name "*.py" | xargs -P 4 python -m py_compile

    # Limit arguments per command
    find . -name "*.py" | xargs -n 1 rm
```
### watch (Run Command Repeatedly)
```bash
    watch -n 2 "docker ps"              # run every 2 seconds
    watch -n 1 "ls -la /app"          # run every second
    watch -d "ls -la"                  # highlight differences between runs
```
### envsubst (Environment Variable Substitution)
```bash
    # Replace $VAR placeholders in a file with env vars
    export DB_HOST=localhost DB_PASS=secret
    envsubst < cat config.template > config.yml

    # With a specific env file
    envsubst $(grep -v '^#' .env | sed 's/=.*//') < cat template.conf > app.conf
```
### rsync (File Sync)
```bash
    rsync -avz source/ user@server:/dest/    # sync to remote (archive, verbose, compress)
    rsync -avz --delete source/ dest/          # sync + delete extras in dest
    rsync -avz --dry-run source/ dest/       # preview without making changes
    rsync -avz --exclude='node_modules' source/ dest/  # exclude patterns
```
### Quick Reference Card

| Task | Command |
|------|---------|
| What's my IP | `curl ifconfig.me` |
| Public IP of a server | `curl -s ifconfig.me` |
| What OS am I on | `cat /etc/os-release` |
| Disk free | `df -h` |
| Memory free | `free -h` |
| CPU info | `lscpu` |
| Last login | `last` |
| Uptime | `uptime` |
| Path to a command | `which python` or `type python` |
| Environment vars | `env` or `printenv` |
| Base64 encode | `echo -n "text" | base64` |
| Base64 decode | `echo "dGV4dA==" | base64 -d` |
| UUID generate | `uuidgen` |
| Random string | `openssl rand -hex 16` |
| Download a file | `wget url` or `curl -O url` |
| Port scan (simple) | `nc -zv host port` |
| Disk I/O stats | `iostat` |
| Network connections | `ss -tunlp` |
| Python version | `python3 --version` |
| Pip packages | `pip list` |
| Node version | `node --version` |
| Docker version | `docker --version` |