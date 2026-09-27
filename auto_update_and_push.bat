@echo off
set GIT="C:\Users\Admin\.gemini\antigravity\scratch\mingit\cmd\git.exe"
set DIR="C:\Users\Admin\.gemini\antigravity\scratch\memscript-ai"
set CSC="C:\Windows\Microsoft.NET\Framework64\v4.0.30319\csc.exe"

echo [1/3] Compiling GUI as main app (memscript-ai.exe)...
%CSC% /target:winexe /r:System.Windows.Forms.dll /r:System.Drawing.dll /out:"%DIR%\memscript-ai.exe" /optimize+ "%DIR%\Gui.cs"
%CSC% /target:exe /out:"%DIR%\memscript-cli.exe" /optimize+ "%DIR%\Program.cs"

echo [2/3] Adding files to Git...
%GIT% -C %DIR% add .
%GIT% -C %DIR% commit -m "Set GUI as primary memscript-ai.exe application"

echo [3/3] Pushing to GitHub...
%GIT% -C %DIR% push origin main
echo Done!
