# Aliases
alias ..='cd ..'
alias ...='cd ../..'
alias greph='history | grep '
alias fh='history --max=500 --reverse -w /dev/stdout | fzf --height=50% --preview="eval (echo {})" | string replace "\n" "" | source'
alias install='sudo dnf install --setopt=tsflags=nodocs --setopt=install_weak_deps=False'
alias ls='ls -CF --color=auto'
starship init fish | source

set -x STUDIO_JDK "/usr/java/jdk-18.0.2.1/bin"

set -x JAVA_HOME "/mnt/Work/JAVA/jdk-18.0.2.1_linux-x64_bin/jdk-18.0.2.1"
alias studio="bash /mnt/e9bf59ee-d1a4-402b-8569-388a82d48960/android_studio/android-studio-2023.1.1.28-linux/android-studio/bin/studio.sh"
