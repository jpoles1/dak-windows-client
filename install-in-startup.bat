copy "target\release\dak-windows-client.exe" "%userprofile%\AppData\Roaming\Microsoft\Windows\Start Menu\Programs\Startup"
md "%userprofile%\AppData\Roaming\Microsoft\Windows\Start Menu\Programs\Startup\dak-util\"
copy /Y "dak-util\*" "%userprofile%\AppData\Roaming\Microsoft\Windows\Start Menu\Programs\Startup\dak-util"
PAUSE