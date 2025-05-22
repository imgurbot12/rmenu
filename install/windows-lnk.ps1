
$CONFIG="$HOME\AppData\Roaming\rmenu\rmenu.exe"
$SHORTCUTS="$HOME\AppData\Roaming\Microsoft\Windows\Start Menu\Programs\rmenu"

$null = New-Item -ItemType Directory -Path $SHORTCUTS -Force

$shell = New-Object -ComObject WScript.Shell
$shortcut = $shell.CreateShortcut("$SHORTCUTS\RMenu Search.lnk")
$shortcut.TargetPath = $CONFIG
$shortcut.Arguments  = "-r search"
$shortcut.Save()
