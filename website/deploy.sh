#!/bin/bash
# Deployment script for LLM Cost Ops Documentation Site

set -e  # Exit on error

echo "================================"
echo "LLM Cost Ops - Documentation Deploy"
echo "================================"

# Check if we're in the right directory
if [ ! -f "package.json" ]; then
    echo "Error: Must run from website directory"
    exit 1
fi

# Detect deployment target
DEPLOY_TARGET=${DEPLOY_TARGET:-"static"}

echo "Deploy target: $DEPLOY_TARGET"

# Clean previous build
echo "Cleaning previous build..."
npm run clear || true
rm -rf build/

# Install dependencies
echo "Installing dependencies..."
npm ci

# Run type check
echo "Running type check..."
npm run typecheck

# Build the site
echo "Building site..."
npm run build

case "$DEPLOY_TARGET" in
    "static")
        echo "Static build complete! Files are in build/"
        echo "To test locally: npm run serve"
        ;;

    "github-pages")
        echo "Deploying to GitHub Pages..."
        GIT_USER=$(git config user.name) \
        USE_SSH=true \
        npm run deploy
        ;;

    "netlify")
        echo "Build complete for Netlify!"
        echo "Netlify will automatically deploy from build/"
        ;;

    "vercel")
        echo "Build complete for Vercel!"
        echo "Vercel will automatically deploy from build/"
        ;;

    "s3")
        echo "Deploying to AWS S3..."
        if [ -z "$AWS_S3_BUCKET" ]; then
            echo "Error: AWS_S3_BUCKET environment variable not set"
            exit 1
        fi
        aws s3 sync build/ "s3://$AWS_S3_BUCKET/" --delete
        echo "Deployed to S3: $AWS_S3_BUCKET"
        ;;

    *)
        echo "Unknown deploy target: $DEPLOY_TARGET"
        echo "Valid targets: static, github-pages, netlify, vercel, s3"
        exit 1
        ;;
esac

echo "================================"
echo "Deployment complete!"
echo "================================"
