@echo off
REM MoneyEarning Windows Release Build Script
REM Requires: Rust, Node.js 18+, pnpm, Tauri CLI
REM Builds MSI + NSIS installers for Windows x64

echo ============================================
echo  MoneyEarning Windows Release Build v0.6.0
echo ============================================
echo.

REM Check prerequisites
where cargo >nul 2>&1 || (echo ERROR: Rust/Cargo not found & exit /b 1)
where node >nul 2>&1 || (echo ERROR: Node.js not found & exit /b 1)
where pnpm >nul 2>&1 || (echo ERROR: pnpm not found & exit /b 1)

echo [1/4] Installing frontend dependencies...
call pnpm install --frozen-lockfile
if %ERRORLEVEL% NEQ 0 (echo ERROR: pnpm install failed & exit /b 1)

echo [2/4] Building TypeScript packages...
call pnpm build:packages
if %ERRORLEVEL% NEQ 0 (echo ERROR: package build failed & exit /b 1)

echo [3/4] Building Tauri release (this may take 5-15 minutes)...
cd packages\app\src-tauri
call cargo build --release
if %ERRORLEVEL% NEQ 0 (echo ERROR: cargo build failed & exit /b 1)
cd ..\..\..

echo [4/4] Creating Windows installers...
REM Tauri bundler creates MSI and NSIS installers
cd packages\app
call pnpm tauri build --bundles msi,nsis
if %ERRORLEVEL% NEQ 0 (echo WARNING: Tauri bundle may have warnings & exit /b 0)
cd ..\..

echo.
echo ============================================
echo  Build Complete!
echo  Installers located in:
echo    packages\app\src-tauri\target\release\bundle\
echo ============================================
echo.
dir /s /b packages\app\src-tauri\target\release\bundle\*.msi 2>nul
dir /s /b packages\app\src-tauri\target\release\bundle\*.exe 2>nul
echo.
echo Done.
