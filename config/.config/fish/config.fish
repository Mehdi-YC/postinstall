# Aliases
alias ..='cd ..'
alias ...='cd ../..'
alias fh='history --max=500 --reverse -w /dev/stdout | fzf --height=50% --preview="eval (echo {})" | string replace "\n" "" | source'
alias install='sudo dnf install --setopt=tsflags=nodocs --setopt=install_weak_deps=False'
alias ls='ls -CF --color=auto'
starship init fish | source
