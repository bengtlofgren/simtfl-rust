#!/bin/bash

# List of platforms to test
PLATFORMS=("alpine" "ubuntu" "debian" "fedora")


# Build all images
for platform in "${PLATFORMS[@]}"; do
    echo "Building $platform image..."
    docker build -f "Dockerfile.$platform" -t "demo-$platform" .
done

# Run all containers and capture outputs
for platform in "${PLATFORMS[@]}"; do
    echo "Running $platform container..."
    docker run --name "run-$platform" "demo-$platform" > "${platform}_output.log" 2>&1
done

# Clean up containers
for platform in "${PLATFORMS[@]}"; do
    docker rm "run-$platform"
done

# Compare all outputs against alpine (or choose another reference)
REFERENCE="alpine_output.log"
for platform in "${PLATFORMS[@]}"; do
    if [ "$platform" != "alpine" ]; then
        echo "Comparing alpine vs $platform:"
        if diff -q "$REFERENCE" "${platform}_output.log" >/dev/null ; then
            echo "✅ $platform output matches reference"
        else
            echo "❌ $platform output differs:"
            diff "$REFERENCE" "${platform}_output.log"
        fi
    fi
done