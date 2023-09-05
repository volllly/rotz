#!/bin/sh

set -e

if ! command -v unzip >/dev/null; then
	echo "Error: unzip is required to install Rotz." 1>&2
	exit 1
fi

if [ "$OS" = "Windows_NT" ]; then
	target="x86_64-pc-windows-msvc"
else
	case $(uname -sm) in
	"Darwin x86_64") target="x86_64-apple-darwin" ;;
	"Darwin arm64") target="aarch64-apple-darwin" ;;
	"Linux aarch64") target="aarch64-unknown-linux-gnu" ;;
  # TOTO: Add support for musl
	*) target="x86_64-unknown-linux-gnu" ;;
	esac
fi

if [ $# -eq 0 ]; then
	rotz_uri="https://github.com/volllly/rotz/releases/latest/download/rotz-${target}.zip"
else
	rotz_uri="https://github.com/volllly/rotz/releases/download/${1}/rotz-${target}.zip"
fi

rotz_install="${ROTZ_INSTALL:-$HOME/.rotz}"
bin_dir="$rotz_install/bin"
exe="$bin_dir/rotz"

if [ ! -d "$bin_dir" ]; then
	mkdir -p "$bin_dir"
fi

curl --fail --location --progress-bar --output "$exe.zip" "$rotz_uri"
unzip -d "$bin_dir" -o "$exe.zip"
chmod +x "$exe"
rm "$exe.zip"

echo "Rotz was installed successfully to $exe"
if command -v rotz >/dev/null; then
	echo "Run 'rotz --help' to get started"
else
	case $SHELL in
	/bin/zsh) shell_profile=".zshrc" ;;
	*) shell_profile=".bashrc" ;;
	esac
	echo "Manually add the directory to your \$HOME/$shell_profile (or similar)"
	echo "  export ROTZ_INSTALL=\"$rotz_install\""
	echo "  export PATH=\"\$ROTZ_INSTALL/bin:\$PATH\""
	echo "Run '$exe --help' to get started"
fi