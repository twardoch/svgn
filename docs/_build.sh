#!/bin/bash
# Build script for SVGN documentation site
# Builds CSS, optimizes assets, and serves Jekyll site

set -e

echo "🚀 Building SVGN Documentation Site"

# Change to docs directory
cd "$(dirname "$0")"

# Install dependencies if node_modules doesn't exist
if [ ! -d "node_modules" ]; then
    echo "📦 Installing Node.js dependencies..."
    npm install
fi

# Build CSS
echo "🎨 Building CSS with PostCSS and Tailwind..."
npm run build-postcss-prod

# Update browserslist if needed
echo "🔄 Updating browserslist..."
npx update-browserslist-db

# Build Jekyll site
echo "🔨 Building Jekyll site..."
bundle exec jekyll build

echo "✅ Build complete! Site built to _site/"
echo "💡 To serve locally: bundle exec jekyll serve"