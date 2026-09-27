@echo off
set GIT="C:\Users\Admin\.gemini\antigravity\scratch\mingit\cmd\git.exe"
set DIR="C:\Users\Admin\.gemini\antigravity\scratch\memscript-ai"

%GIT% -C %DIR% config user.name "Gdfjffghhjkgghhhh"
%GIT% -C %DIR% config user.email "admin@memscript.local"
%GIT% -C %DIR% branch -M main
%GIT% -C %DIR% remote add origin https://github.com/Gdfjffghhjkgghhhh/Don-Dep-Ram-AI.git 2>nul
%GIT% -C %DIR% add .
%GIT% -C %DIR% commit -m "Auto-commit: Full MemScript-AI engine with built-in .exe"
%GIT% -C %DIR% status
