@echo off
echo ============================================
echo    🚀 Building Pixardis Web IDE
echo ============================================

echo.
echo 📁 Changing to web directory...
cd web

echo.
echo 🧹 Cleaning previous build...
if exist pkg rmdir /s /q pkg
if exist ..\pkg rmdir /s /q ..\pkg

echo.
echo 📦 Building WASM package...
wasm-pack build --target web --out-dir ../pkg

if %ERRORLEVEL% neq 0 (
    echo.
    echo ❌ WASM build failed!
    cd ..
    pause
    exit /b 1
)

echo.
echo ✅ WASM build successful!

cd ..

echo.
echo 📁 Creating frontend/pkg directory...
if not exist frontend mkdir frontend
if not exist frontend\pkg mkdir frontend\pkg

echo.
echo 📋 Copying WASM files to frontend/pkg/...
copy pkg\*.* frontend\pkg\

echo.
echo 📁 Files copied to frontend/pkg/:
dir frontend\pkg\ /B

echo.
echo 🌐 To run the web IDE:
echo    1. Make sure your index.html is in frontend/
echo    2. Start a web server from the root:
echo       python -m http.server 8000
echo    3. Open http://localhost:8000/frontend/
echo.
echo    Or serve directly from frontend/:
echo       cd frontend
echo       python -m http.server 8000
echo       Open http://localhost:8000/
echo.

pause