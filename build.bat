@REM Bash script to compile in release mode and pack it in a zip with the assets

@echo off

@REM Compile in release mode
call cargo build --release

@REM Create the zip
call powershell Compress-Archive -Path .\target\release\*.exe, .\assets -DestinationPath release.zip -Force

@REM Echo the size of the zip
call dir release.zip

@REM Done!