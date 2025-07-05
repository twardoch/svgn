#!/bin/bash
# Development server script for SVGN documentation site
# Watches CSS changes and serves Jekyll site

set -e

echo "🌐 Starting SVGN Documentation Development Server"

# Change to docs directory
cd "$(dirname "$0")"

# Start CSS watch in background
echo "👀 Starting PostCSS watch process..."
npm run build-postcss &
CSS_PID=$!

# Start Jekyll serve
echo "🚀 Starting Jekyll server..."
bundle exec jekyll serve --livereload --host 0.0.0.0 --port 4000 &
JEKYLL_PID=$!

# Function to cleanup background processes
cleanup() {
    echo "🛑 Stopping development server..."
    kill $CSS_PID $JEKYLL_PID 2>/dev/null || true
    exit 0
}

# Set trap to cleanup on script exit
trap cleanup INT TERM

echo "✅ Development server running!"
echo "📍 Site available at: http://localhost:4000"
echo "🎨 CSS watching for changes"
echo "🔄 Jekyll watching for changes with LiveReload"
echo "🛑 Press Ctrl+C to stop"

# Wait for background processes
wait