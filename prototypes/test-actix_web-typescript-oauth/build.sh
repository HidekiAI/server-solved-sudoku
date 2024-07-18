#!/bin/bash

npm install dotenv

source .env.sh

npm run build

cargo build
cp *.html target/debug/
cp script.js target/debug/

echo "Build completed"
echo "#####################################################"
echo "Type: cargo run (Press Ctrl+C to stop)"
echo "Open: http://localhost:8080"
echo "Optionally hit F12 in browser to monitor 'Network' traffic prior to clicking 'Login' button"
echo "#####################################################"