@echo off
set GIT="C:\Users\Admin\.gemini\antigravity\scratch\mingit\cmd\git.exe"
set DIR="C:\Users\Admin\.gemini\antigravity\scratch\memscript-ai"

%GIT% -C %DIR% remote set-url origin https://Mnhatrealz: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@github.com/Mnhatrealz/Don-Dep-Ram-AI.git
%GIT% -C %DIR% branch -M main
%GIT% -C %DIR% push -u origin main --force
