
$ErrorActionPreference = "Stop"

#** Variables **#

$CARGO="cargo"
$CARGO_FLAGS="--release "

$CONFIG="$HOME\AppData\Roaming\rmenu"

#** Functions **#

function Deploy {
  param ([String] $dest)
  New-Item -ItemType Directory -Path $dest\plugins\css -Force
  # copy css/theme resources
  Copy-Item -Path themes -Destination $dest\. -Force -Recurse
  Copy-Item -Path plugins\emoji\css\* -Destination $dest\plugins\css
  # copy misc plugins
  Copy-Item -Path plugins\misc\css\* -Destination $dest\plugins\css\.
  Copy-Item -Path plugins\misc\powermenu.ps1 -Destination $dest\plugins\.
  # copy binaries
  Copy-Item -Path target\release\rmenu.exe       -Destination $dest\rmenu.exe
  Copy-Item -Path target\release\rmenu-build.exe -Destination $dest\rmenu-build.exe
  Copy-Item -Path target\release\desktop.exe     -Destination $dest\plugins\rmenu-desktop.exe
  Copy-Item -Path target\release\emoji.exe       -Destination $dest\plugins\rmenu-emoji.exe
  Copy-Item -Path target\release\files.exe       -Destination $dest\plugins\rmenu-files.exe
  Copy-Item -Path target\release\run.exe         -Destination $dest\plugins\rmenu-run.exe
  Copy-Item -Path target\release\search.exe      -Destination $dest\plugins\rmenu-search.exe
  # copy config instance and set default style
  Copy-Item -Path examples\configs\windows.yaml $dest\config.yaml
  Copy-Item -Path $dest\themes\dark.css -Destination $dest\style.css
}

function Compile-All {
  # compile rmenu and rmenu-build
  Invoke-Expression "$CARGO build -p rmenu $CARGO_FLAGS"
	Invoke-Expression "$CARGO build -p rmenu-plugin --bin rmenu-build $CARGO_FLAGS"
  # compile plugins
  Invoke-Expression "$CARGO build -p desktop $CARGO_FLAGS"
  Invoke-Expression "$CARGO build -p emoji   $CARGO_FLAGS"
  Invoke-Expression "$CARGO build -p files   $CARGO_FLAGS"
  Invoke-Expression "$CARGO build -p run     $CARGO_FLAGS"
  Invoke-Expression "$CARGO build -p search  $CARGO_FLAGS"
}

#** Init **#

Compile-All
$null = Deploy $CONFIG

$Env:Path += ";$CONFIG"
