@echo off
set GIT="C:\Users\Admin\.gemini\antigravity\scratch\mingit\cmd\git.exe"
set DIR="C:\Users\Admin\.gemini\antigravity\scratch\memscript-ai"
set CSC="C:\Windows\Microsoft.NET\Framework64\v4.0.30319\csc.exe"

echo [1/3] Compiling CLI (memscript-ai.exe) and GUI (memscript-gui.exe)...
%CSC% /target:exe /out:"%DIR%\memscript-ai.exe" /optimize+ "%DIR%\Program.cs"
%CSC% /target:winexe /r:System.Windows.Forms.dll /r:System.Drawing.dll /out:"%DIR%\memscript-gui.exe" /optimize+ "%DIR%\Gui.cs"

echo [2/3] Adding files to Git...
%GIT% -C %DIR% add .
%GIT% -C %DIR% commit -m "Add modern Dark Mode GUI application"

echo [3/3] Pushing to GitHub...
%GIT% -C %DIR% push origin main
echo Done!
