# .bashrc

# Source global definitions
if [ -f /etc/bashrc ]; then
	. /etc/bashrc
fi
# User specific environment
if ! [[ "$PATH" =~ "$HOME/.local/bin:$HOME/bin:" ]]
then
    PATH="$HOME/.local/bin:$HOME/bin::$HOME/.config/bin:$PATH"
fi
export PATH
#unset rc
#. "$HOME/.cargo/env"


# Don't put duplicate lines in the history and do not add lines that start with a space
export HISTCONTROL=erasedups:ignoredups:ignorespac:ignoreboth
# Causes bash to append to history instead of overwriting it so if you start a new terminal, you have old session history
shopt -s histappend
PROMPT_COMMAND="history -n; history -w; history -c; history -r; $PROMPT_COMMAND"
# Ignore case on auto-completion
bind "set completion-ignore-case on"
# Show auto-completion list automatically, without double tab
bind "set show-all-if-ambiguous On"



# fzf
export FZF_DEFAULT_COMMAND='fd --type f --color=never --hidden'
export FZF_DEFAULT_OPTS='--no-height --color=bg+:#343d46,gutter:-1,pointer:#ff3c3c,info:#0dbc79,hl:#0dbc79,hl+:#23d18b'

export FZF_CTRL_T_COMMAND="$FZF_DEFAULT_COMMAND"
export FZF_CTRL_T_OPTS="--preview 'bat --color=always --line-range :50 {}'"

export FZF_ALT_C_COMMAND='fd --type d . --color=never --hidden'
export FZF_ALT_C_OPTS="--preview 'tree -C {} | head -50'"


eval "$(starship init bash)"

# including aliases and functions

#Aliases :
alias ..='cd ..'
alias ...='cd ../..'
alias greph='history | grep '
alias fh='history --max=500 --reverse -w /dev/stdout | fzf --height=50% --preview="eval (echo {})" | string replace "\n" "" | source'
alias ls="ls -CF --color=auto"

#Functions :
mkcd () { mkdir -p "$@" && eval cd "\"\$$#\""; }
