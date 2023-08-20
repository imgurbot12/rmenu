#!/bin/sh

SELF=`realpath $0`
THEME=`realpath "$(dirname $0)/themes/powermenu.css"`
RMENU=${RMENU:-"rmenu"}

options() {
  rmenu-build options \
  -t $THEME \
  -n ArrowRight -p ArrowLeft \
  -w 550 -h 150 -M 0
}

#: desc => generate confirmation entries
#: usage => $cmd $name?
confirm() {
  cmd=$1
  name="${2:-"Confirm"}"
  options
  rmenu-build entry -n "Cancel" -I "" -a "`rmenu-build action -m echo "$name Cancelled"`"
  rmenu-build entry -n "$name" -I "" -a "`rmenu-build action "$cmd"`"
}

#: desc => generate non-confirm entry
#: usage => $icon $name $cmd
gen_direct() {
  rmenu-build entry -n "$2" -I "$1" -a "`rmenu-build action "$3"`"
}

#: desc  => generate confirmation entry
#: usage => $icon $name $cmd 
gen_confirm() {
  rmenu-build entry -n "$2" -I "$1" -a "`rmenu-build action "$SELF confirm '$2:$3'"`"
}

#: desc => generate action-entry
#: usage => $icon $name $command $do-confirm
action() {
  icon="$1"
  name="$2"
  cmd="$3"
  confirm="$4"
  [ -z "$confirm" ] \
    && gen_direct "$icon" "$name" "$cmd" \
    || gen_confirm "$icon" "$name" "$cmd"
}

case "$1" in
  "list")
    confirm="$2"
    options
    action "⏻" "Shutdown" "systemctl poweroff" "$2"
    action "" "Reboot"   "systemctl reboot"   "$2"
    action "⏾" "Suspend"  "systemctl suspend"  "$2"
    action "" "Log Out"  "sway exit"          "$2"
    ;;
  "confirm")
    name=`echo $2 | cut -d ':' -f1`
    action=`echo $2 | cut -d ':' -f2`
    confirm "$action" "$name" | $RMENU
    ;;
  *)
    echo "usage: $0 <list|confirm> <args...>" && exit 1
    ;;
esac
