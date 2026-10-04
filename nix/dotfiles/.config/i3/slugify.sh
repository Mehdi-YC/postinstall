# now we have to install xdotool for this 
str=$(xclip -o -selection clipboard);echo ${str,,} | iconv -t ascii//TRANSLIT | sed "s/d'//g" |tr  ' ' '_' | xclip -selection clipboard
