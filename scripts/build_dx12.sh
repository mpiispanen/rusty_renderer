#!/bin/bash
# Build DirectX 12 backend using distrobox container on Bazzite

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
CONTAINER_NAME="fedora-dev"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${GREEN}Building DirectX 12 backend...${NC}"

# Check if distrobox exists
if ! command -v distrobox &> /dev/null; then
    echo -e "${RED}Error: distrobox not found${NC}"
    echo "Distrobox should be installed on Bazzite by default"
    exit 1
fi

# Check if container exists
if ! distrobox list | grep -q "^$CONTAINER_NAME"; then
    echo -e "${YELLOW}Container '$CONTAINER_NAME' not found${NC}"
    echo "Creating Fedora development container..."
    echo "This will take a few minutes on first run..."
    
    distrobox create --name "$CONTAINER_NAME" --image fedora:41
    
    echo -e "${GREEN}Installing build tools in container...${NC}"
    distrobox enter "$CONTAINER_NAME" -- bash -c "
        sudo dnf install -y mingw64-gcc mingw64-winpthreads-static rustup && \
        rustup-init -y && \
        source \$HOME/.cargo/env && \
        rustup target add x86_64-pc-windows-gnu
    "
    
    echo -e "${GREEN}Container setup complete!${NC}"
fi

# Build the project
echo -e "${GREEN}Compiling for Windows target...${NC}"
distrobox enter "$CONTAINER_NAME" -- bash -c "
    source \$HOME/.cargo/env && \
    cd '$PROJECT_DIR' && \
    cargo build --release --target x86_64-pc-windows-gnu
"

if [ $? -eq 0 ]; then
    echo -e "${GREEN}Build successful!${NC}"
    echo "Windows binary at: target/x86_64-pc-windows-gnu/release/rusty_renderer.exe"
    echo ""
    echo "To run with DirectX 12 via Proton:"
    echo "  ./scripts/test_dx12_proton.sh --release"
else
    echo -e "${RED}Build failed${NC}"
    exit 1
fi
