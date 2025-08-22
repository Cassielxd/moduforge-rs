@echo off
setlocal enabledelayedexpansion

echo Starting to publish all crates...

set "CRATES=model transform state core macro derive expression template engine collaboration collaboration_client file persistence search"

for %%c in (%CRATES%) do (
    echo Publishing: %%c
    cd crates\%%c
    cargo publish --registry crates-io
    if !errorlevel! neq 0 (
        echo Failed to publish: %%c
        exit /b !errorlevel!
    )
    cd ..\..
    echo Successfully published: %%c
    :: Wait 5 seconds for crates.io to update
    timeout /t 10 /nobreak > nul
)

echo All crates have been published!
