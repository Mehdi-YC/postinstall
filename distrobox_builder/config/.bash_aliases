#Aliases
alias ..='cd ..'
alias ...='cd ../..'
alias greph='history | grep '
alias fh='history --max=500 --reverse -w /dev/stdout | fzf --height=50% --preview="eval (echo {})" | string replace "\n" "" | source'
alias ls="ls -CF --color=auto"

