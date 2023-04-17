mkcd () { mkdir -p "$@" && eval cd "\"\$$#\""; }

dbox () { 
    if [ -d "$HOME/db_$1" ];then
        echo "Distrobox already exists $1 entering $1 ..."
        distrobox enter $1
    else 
        distrobox create -H $HOME/db_$1 $1 -i localhost/db_alpine
        distrobox enter $1
    fi
}
