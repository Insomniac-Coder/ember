@echo off
rem usage: cl_build.bat <source> <out.exe>   (C or C++ by extension; Ember's release /O2 /MD)
call "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat" >nul 2>nul
if /I "%~x1"==".cpp" (
  cl /nologo /O2 /MD /fp:precise /EHsc /std:c++17 %1 /Fe:%2 /Fo:%~dp2 >nul
) else (
  cl /nologo /O2 /MD /fp:precise /std:c11 %1 /Fe:%2 /Fo:%~dp2 >nul
)
