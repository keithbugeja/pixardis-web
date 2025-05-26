#!/bin/bash
# filepath: /Users/keith/Development/pixardis-web/build-web.sh

echo "============================================"
echo "   🚀 Building Pixardis Web IDE"
echo "============================================"

echo
echo "📁 Changing to web directory..."
cd web

echo
echo "🧹 Cleaning previous build..."
rm -rf pkg
rm -rf ../pkg

echo
echo "📦 Building WASM package..."
wasm-pack build --target web --out-dir ../pkg

if [ $? -ne 0 ]; then
    echo
    echo "❌ WASM build failed!"
    cd ..
    exit 1
fi

echo
echo "✅ WASM build successful!"

cd ..

echo
echo "📁 Creating frontend/pkg directory..."
mkdir -p frontend/pkg

echo
echo "📋 Copying WASM files to frontend/pkg/..."
cp pkg/* frontend/pkg/

echo
echo "📁 Files copied to frontend/pkg/:"
ls -1 frontend/pkg/

echo
echo "🌐 To run the web IDE:"
echo "   1. Make sure your index.html is in frontend/"
echo "   2. Start a web server from the root:"
echo "      python3 -m http.server 8000"
echo "   3. Open http://localhost:8000/frontend/"
echo
echo "   Or serve directly from frontend/:"
echo "      cd frontend"
echo "      python3 -m http.server 8000"
echo "      Open http://localhost:8000/"
echo

read -p "Press any key to continue..."