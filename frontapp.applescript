use framework "Foundation"
use framework "AppKit"
use scripting additions

tell application "System Events"
	set frontApp to first application process whose frontmost is true
	set frontAppName to name of frontApp
	set frontAppPID to unix id of frontApp
	set windowTitle to ""
	try
		tell frontApp to set windowTitle to name of first window
	end try
end tell

set nsapp to current application's NSRunningApplication's runningApplicationWithProcessIdentifier:frontAppPID
set appURL to nsapp's bundleURL()
set appPath to (appURL's |path|()) as text

set ws to current application's NSWorkspace's sharedWorkspace()
set img to ws's iconForFile:appPath
img's setSize:{128, 128}

set tiffData to img's TIFFRepresentation()
set rep to current application's NSBitmapImageRep's imageRepWithData:tiffData
set pngData to rep's representationUsingType:(current application's NSBitmapImageFileTypePNG) |properties|:(current application's NSDictionary's dictionary())
set b64 to (pngData's base64EncodedStringWithOptions:0) as text

set NUL to (ASCII character 0)
return frontAppName & NUL & frontAppPID & NUL & windowTitle & NUL & b64
