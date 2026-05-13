set appPath to POSIX path of (path to me)
set binPath to appPath & "Contents/MacOS/presencejj"

tell application "Terminal"
    activate
    do script quoted form of binPath
end tell