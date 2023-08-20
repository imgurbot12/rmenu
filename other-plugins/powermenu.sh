#!/bin/sh

SELF=`realpath $0`
THEME=`realpath "$(dirname $0)/themes/powermenu.css"`
RMENU=${RMENU:-"rmenu"}

#: desc => generate options for basic operation
main_options() {
  rmenu-build options \
  -t $THEME \
  -n ArrowRight -p ArrowLeft \
  -w 550 -h 150 -M 0
}

#: desc => generate options for confirm operation
confirm_options() {
  rmenu-build options \
  -t $THEME \
  -n ArrowRight -p ArrowLeft \
  -w 300 -h 150 -M 0
}

#: desc => generate confirmation entries
#: usage => $cmd $name?
confirm() {
  cmd=$1
  name="${2:-"Confirm"}"
  confirm_options
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
  "help")
    echo "usage: $0 <args...>" && exit 1
    ;;
  "confirm")
    name=`echo $2 | cut -d ':' -f1`
    action=`echo $2 | cut -d ':' -f2`
    confirm "$action" "$name" | $RMENU
    ;;
  *)
    [ "$1" != "--no-confirm" ] && confirm="Y"
    main_options
    action "⏻" "Shutdown" "systemctl poweroff" "$confirm"
    action "" "Reboot"   "systemctl reboot"   "$confirm"
    action "⏾" "Suspend"  "systemctl suspend"  "$confirm"
    action "" "Log Out"  "sway exit"          "$confirm"
    ;;
esac
