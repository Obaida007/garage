@echo off
set PATH=C:\Users\TECHNO\.cargo\bin;C:\Users\TECHNO\AppData\Local\Programs\Kimi\resources\resources\runtime;%PATH%
cd /d C:\Users\TECHNO\Projects\al-sahil-garage
call node_modules\.bin\tauri.cmd build --config "{\"build\":{\"beforeBuildCommand\":\"\"}}" > build.log 2>&1
echo %ERRORLEVEL% > build.exit
