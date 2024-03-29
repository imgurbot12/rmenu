#!/bin/sh

get_sinks() {
  sinks=`pactl list sinks | grep -e 'Sink #' -e 'Name: ' -e 'Description: ' | nl -s '>'`
  default=`pactl get-default-sink`
  for i in `seq 1 3 $(echo "$sinks" | wc -l)`; do
    sink=`echo "$sinks" | grep "$i>" | cut -d '#' -f2`
    name=`echo "$sinks" | grep "$(expr $i + 1)>" | cut -d ':' -f2 | xargs echo -n`
    desc=`echo "$sinks" | grep "$(expr $i + 2)>" | cut -d ':' -f2 | xargs echo -n`
    if [ "$name" = "$default" ]; then
      desc="* $desc"
    fi
    rmenu-build entry -n "$desc" -a "`rmenu-build action "pactl set-default-sink $sink"`"
  done
}

get_sinks
