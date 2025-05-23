param ([String] $command)

$CSS   = $(Resolve-Path -Path "$PSScriptRoot/css/powermenu.css")
$RMENU = if ($env:RMENU) { $env:RMENU } else { "rmenu" };

$CMD_PREFIX = "cmd /c"
$SLEEP_COMMAND = "rundll32.exe powrprof.dll, SetSuspendState Sleep"

Function Main-Options {
  rmenu-build options `
    -C $CSS `
    -n ArrowRight -p ArrowLeft `
    -W 550 -H 150 -M 0
}

Function Confirm-Options {
  rmenu-build options `
    -C $CSS `
    -n ArrowRight -p ArrowLeft `
    -W 300 -H 150 -M 0
}

Function Confirm {
  param ([String] $command, [String] $name = "Confirm")
  Confirm-Options
  rmenu-build entry `
    -n 'Cancel' -I '' `
    -a $(rmenu-build action -m echo "$name Cancelled" --base64)
  rmenu-build entry `
    -n "$name" -I "" `
    -a $(rmenu-build action "$CMD_PREFIX $command" --base64)
}

Function Gen-Direct {
  param ([String] $icon, [String] $name, [String] $command)
  rmenu-build entry `
    -n "$name" -I "$icon" `
    -a $(rmenu-build action "$CMD_PREFIX $command" --base64)
}

Function Gen-Confirm {
  param ([String] $icon, [String] $name, [String] $command)
  $inner   = [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes("${name}:$command"))
  $command = "powershell -WindowStyle hidden -Command '$PSCommandPath confirm $inner'"
  rmenu-build entry `
    -n "$name" -I "$icon" `
    -a $(rmenu-build action "$command" --base64)
}

Function Action {
  param ([String] $icon, [String] $name, [String] $cmd, [String] $confirm)
  if ($confirm) { Gen-Confirm "$icon" "$name" "$cmd" }
  else { Gen-Direct "$icon" "$name" "$cmd" }
}

switch ($command) {
  "help" {
    Write-Host "Usage: $PSCommandPath <args...>"
    Exit 1
  }
  "confirm" {
    $cmd=[Text.Encoding]::Utf8.GetString([Convert]::FromBase64String($args[0]))
    $items=$cmd.split(':')
    $name=$items[0]
    $action=$items[1]
    Confirm "$action" "$name" | & "$RMENU"
  }
  default {
    if ($command -ne '--no-confirm') { $confirm="Y" }
    Main-Options
    Action "⏻" "Shutdown" "shutdown /s /f" "$confirm"
    Action "" "Reboot"   "shutdown /r /f" "$confirm"
    Action "⏾" "Suspend"  "$SLEEP_COMMAND" "$confirm"
    Action "" "Log Out"  "shutdown /l"    "$confirm"
  }
}
