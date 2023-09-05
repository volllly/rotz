#!/usr/bin/env pwsh

$ErrorActionPreference = 'Stop'

if ($v) {
  $Version = "v${v}"
}
if ($Args.Length -eq 1) {
  $Version = $Args.Get(0)
}

$RotzInstall = $env:ROTZ_INSTALL
$BinDir = if ($RotzInstall) {
  "${RotzInstall}\bin"
} else {
  "${Home}\.rotz\bin"
}

$RotzZip = "$BinDir\rotz.zip"
$RotzExe = "$BinDir\rotz.exe"
$Target = if([Environment]::Is64BitOperatingSystem) {
  'x86_64-pc-windows-msvc'
} else {
  'i686-pc-windows-msvc'
}

$DownloadUrl = if (!$Version) {
  "https://github.com/volllly/rotz/releases/latest/download/rotz-${Target}.zip"
} else {
  "https://github.com/volllly/rotz/releases/download/${Version}/rotz-${Target}.zip"
}

if (!(Test-Path $BinDir)) {
  New-Item $BinDir -ItemType Directory | Out-Null
}

curl.exe -Lo $RotzZip $DownloadUrl

tar.exe xf $RotzZip -C $BinDir

Remove-Item $RotzZip

$User = [System.EnvironmentVariableTarget]::User
$Path = [System.Environment]::GetEnvironmentVariable('Path', $User)
if (!(";${Path};".ToLower() -like "*;${BinDir};*".ToLower())) {
  [System.Environment]::SetEnvironmentVariable('Path', "${Path};${BinDir}", $User)
  $Env:Path += ";${BinDir}"
}

Write-Output "Rotz was installed successfully to ${RotzExe}"
Write-Output "Run 'rotz --help' to get started"