@echo off


set PATH=C:\Users\interlink\.cargo\bin;C:\Users\interlink\AppData\Local\Programs\Kimi\resources\resources\runtime;%PATH%
cd /d C:\Users\interlink\garage
call npm run tauri:build  > build.log 2>&1
echo %ERRORLEVEL% > build.exit
