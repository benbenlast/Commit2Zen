#!/bin/bash
export MSVC_BIN="/c/Program Files (x86)/Microsoft Visual Studio/18/BuildTools/VC/Tools/MSVC/14.50.35717/bin/Hostx64/x64"
export MSVC_LIB="/c/Program Files (x86)/Microsoft Visual Studio/18/BuildTools/VC/Tools/MSVC/14.50.35717/lib/x64"
export MSVC_INCLUDE="/c/Program Files (x86)/Microsoft Visual Studio/18/BuildTools/VC/Tools/MSVC/14.50.35717/include"
export SDK_LIB="/c/Program Files (x86)/Windows Kits/10/Lib/10.0.26100.0/um/x64"
export SDK_UCRT_LIB="/c/Program Files (x86)/Windows Kits/10/Lib/10.0.26100.0/ucrt/x64"
export SDK_INCLUDE="/c/Program Files (x86)/Windows Kits/10/Include/10.0.26100.0"

export PATH="$MSVC_BIN:$PATH"
export LIB="$MSVC_LIB;$SDK_LIB;$SDK_UCRT_LIB"
export INCLUDE="$MSVC_INCLUDE;$SDK_INCLUDE"

cd "E:/AI/Commit2Zen-master"
npm run tauri dev