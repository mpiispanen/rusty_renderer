#!/bin/bash
# Script to populate reference images from CI artifacts
#
# Usage: ./scripts/populate_references.sh <artifact-zip>
#
# Downloads and extracts reference images from CI artifacts into the
# references/ directory and commits them with proper LFS tracking.

set -e

if [ $# -lt 1 ]; then
    echo "Usage: $0 <artifact-zip-or-directory>"
    echo ""
    echo "Examples:"
    echo "  $0 visual-regression-report-all-backends.zip"
    echo "  $0 screenshots/"
    exit 1
fi

SOURCE="$1"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$REPO_ROOT"

# Ensure LFS is initialized
if ! git lfs version > /dev/null 2>&1; then
    echo "Error: Git LFS not installed"
    exit 1
fi

git lfs install

# Extract if it's a zip file
if [[ "$SOURCE" == *.zip ]]; then
    echo "Extracting $SOURCE..."
    TEMP_DIR=$(mktemp -d)
    unzip -q "$SOURCE" -d "$TEMP_DIR"
    SOURCE_DIR="$TEMP_DIR/screenshots"
else
    SOURCE_DIR="$SOURCE"
fi

if [ ! -d "$SOURCE_DIR" ]; then
    echo "Error: Screenshots directory not found: $SOURCE_DIR"
    exit 1
fi

echo "Populating reference images from $SOURCE_DIR..."

# Copy triangle references
mkdir -p references/triangle

for backend in vulkan wgpu directx; do
    SCREENSHOT="$SOURCE_DIR/${backend}-triangle.png"
    if [ -f "$SCREENSHOT" ]; then
        cp "$SCREENSHOT" "references/triangle/"
        echo "  ✅ Copied ${backend}-triangle.png"
    else
        echo "  ⚠️  Missing ${backend}-triangle.png"
    fi
done

# Clean up temp directory if we extracted a zip
if [[ "$SOURCE" == *.zip ]]; then
    rm -rf "$TEMP_DIR"
fi

# Check LFS tracking
echo ""
echo "Checking Git LFS tracking..."
git lfs ls-files references/ || echo "No LFS files tracked yet (will be tracked on commit)"

echo ""
echo "✅ Reference images populated!"
echo ""
echo "Next steps:"
echo "  1. Review images: ls -lh references/triangle/"
echo "  2. Update references/triangle/README.md with details"
echo "  3. Stage: git add references/"
echo "  4. Commit: git commit -m 'Add baseline reference images'"
echo "  5. Push: git push origin main"
