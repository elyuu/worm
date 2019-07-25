xrdb -load $HOME/.Xresources &
xeyes &
xclock &
sleep 2 && urxvt &

exec target/debug/worm
