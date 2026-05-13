cargo build --release
cargo bundle --release
FOLDER="target/release/bundle/osx"
APP="$FOLDER/PresenceJJ.app"

mv "$APP/Contents/MacOS/presencejj" "$APP/Contents/MacOS/presencejj-tui"

cat > "$APP/Contents/MacOS/presencejj" <<'SH'
#!/bin/zsh
APP_DIR="$(cd "$(dirname "$0")/.." && pwd)"
BIN="$APP_DIR/MacOS/presencejj-tui"

osascript -e 'tell application "Terminal"
    activate
    do script quoted form of POSIX path of "'"$BIN"'"
end tell'
SH

chmod +x "$APP/Contents/MacOS/presencejj"

codesign --force --deep --sign - $APP
codesign --verify --deep --strict --verbose=2 $APP

ditto -c -k --keepParent $APP "$FOLDER/PresenceJJ-macos.zip"