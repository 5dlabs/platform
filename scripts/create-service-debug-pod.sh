#!/bin/bash
set -euo pipefail

# Script to create a debug pod with the same mount configuration as agent jobs
# Usage: ./create-service-debug-pod.sh [service-name]

SERVICE_NAME="${1:-todo-api-test}"
NAMESPACE="orchestrator"
POD_NAME="debug-${SERVICE_NAME}"

echo "Creating debug pod for service: ${SERVICE_NAME}"

cat <<EOF | kubectl apply -f -
apiVersion: v1
kind: Pod
metadata:
  name: ${POD_NAME}
  namespace: ${NAMESPACE}
  labels:
    app: debug-pod
    service: ${SERVICE_NAME}
spec:
  containers:
  - name: debug
    image: ghcr.io/5dlabs/platform/claude-code:latest
    command: ["sleep", "86400"]  # Sleep for 24 hours
    volumeMounts:
    - name: workspace
      mountPath: /workspace
      subPath: ${SERVICE_NAME}
    workingDir: /workspace
    securityContext:
      runAsUser: 0
      runAsGroup: 0
      runAsNonRoot: false
  volumes:
  - name: workspace
    persistentVolumeClaim:
      claimName: shared-workspace
  nodeSelector:
    kubernetes.io/hostname: talos-a43-ee1
  imagePullSecrets:
  - name: ghcr-secret
EOF

echo "Waiting for pod to be ready..."
kubectl wait --for=condition=Ready pod/${POD_NAME} -n ${NAMESPACE} --timeout=60s

echo ""
echo "Debug pod created successfully!"
echo ""
echo "To access the pod:"
echo "  kubectl exec -it -n ${NAMESPACE} ${POD_NAME} -- bash"
echo ""
echo "The pod has the same workspace mount as the agent jobs:"
echo "  - PVC 'shared-workspace' mounted at /workspace"
echo "  - Using subPath: ${SERVICE_NAME}"
echo "  - Working directory: /workspace"
echo ""
echo "To delete the pod when done:"
echo "  kubectl delete pod ${POD_NAME} -n ${NAMESPACE}"