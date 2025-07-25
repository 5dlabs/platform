#!/bin/bash
# Package 5D Labs Platform CRDs for release

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CHART_DIR="${SCRIPT_DIR}/../charts/orchestrator"
CRDS_DIR="${CHART_DIR}/crds"
OUTPUT_DIR="${1:-${SCRIPT_DIR}/../dist}"

# Ensure output directory exists
mkdir -p "${OUTPUT_DIR}"

echo "🔧 Packaging 5D Labs Platform CRDs..."
echo "📁 Chart directory: ${CHART_DIR}"
echo "📁 CRDs directory: ${CRDS_DIR}"
echo "📁 Output directory: ${OUTPUT_DIR}"

# Check if CRDs directory exists
if [[ ! -d "${CRDS_DIR}" ]]; then
    echo "❌ Error: CRDs directory not found at ${CRDS_DIR}"
    exit 1
fi

# Check if platform-crds.yaml exists
if [[ ! -f "${CRDS_DIR}/platform-crds.yaml" ]]; then
    echo "❌ Error: platform-crds.yaml not found at ${CRDS_DIR}/platform-crds.yaml"
    echo "   Please ensure the combined CRDs file exists."
    exit 1
fi

# Copy the combined CRDs file
cp "${CRDS_DIR}/platform-crds.yaml" "${OUTPUT_DIR}/"

# Validate the CRDs
echo "🔍 Validating CRDs..."
if kubectl apply --dry-run=client -f "${OUTPUT_DIR}/platform-crds.yaml" > /dev/null 2>&1; then
    echo "✅ CRDs validation passed"
else
    echo "❌ CRDs validation failed"
    exit 1
fi

# Generate individual CRD files as well (for flexibility)
echo "📦 Copying individual CRD files..."
cp "${CRDS_DIR}/coderun-crd.yaml" "${OUTPUT_DIR}/"
cp "${CRDS_DIR}/docsrun-crd.yaml" "${OUTPUT_DIR}/"

# Generate checksums
echo "🔐 Generating checksums..."
cd "${OUTPUT_DIR}"
sha256sum platform-crds.yaml > platform-crds.yaml.sha256
sha256sum coderun-crd.yaml > coderun-crd.yaml.sha256
sha256sum docsrun-crd.yaml > docsrun-crd.yaml.sha256

echo "✅ CRDs packaged successfully!"
echo ""
echo "📦 Generated files:"
ls -la "${OUTPUT_DIR}"/*.yaml "${OUTPUT_DIR}"/*.sha256
echo ""
echo "🚀 Upload these files to GitHub releases:"
echo "   - platform-crds.yaml (combined CRDs)"
echo "   - coderun-crd.yaml (individual CRD)"
echo "   - docsrun-crd.yaml (individual CRD)"
echo "   - *.sha256 (checksums)"
echo ""
echo "📋 Installation command for users:"
echo "   kubectl apply -f https://github.com/5dlabs/platform/releases/download/TAG/platform-crds.yaml"