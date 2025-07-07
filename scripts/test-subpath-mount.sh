#!/bin/bash
set -euo pipefail

NAMESPACE="orchestrator"
SERVICE_NAME="todo-api-test"

echo "Testing subPath mount behavior..."

# First, create a job that writes to the full PVC path
cat <<EOF | kubectl apply -f -
apiVersion: batch/v1
kind: Job
metadata:
  name: test-write-full-path
  namespace: $NAMESPACE
spec:
  template:
    spec:
      restartPolicy: Never
      containers:
      - name: writer
        image: busybox
        command: ["/bin/sh", "-c"]
        args:
        - |
          echo "Writing to full PVC path..."
          mkdir -p /workspace/${SERVICE_NAME}
          echo "Hello from full path" > /workspace/${SERVICE_NAME}/test.txt
          ls -la /workspace/
          ls -la /workspace/${SERVICE_NAME}/
        volumeMounts:
        - name: workspace
          mountPath: /workspace
      volumes:
      - name: workspace
        persistentVolumeClaim:
          claimName: shared-workspace
      nodeSelector:
        kubernetes.io/hostname: talos-a43-ee1
EOF

echo "Waiting for write job to complete..."
kubectl wait --for=condition=complete job/test-write-full-path -n $NAMESPACE --timeout=60s

echo ""
echo "Write job logs:"
kubectl logs job/test-write-full-path -n $NAMESPACE

# Now create a job that reads using subPath
cat <<EOF | kubectl apply -f -
apiVersion: batch/v1
kind: Job
metadata:
  name: test-read-subpath
  namespace: $NAMESPACE
spec:
  template:
    spec:
      restartPolicy: Never
      containers:
      - name: reader
        image: busybox
        command: ["/bin/sh", "-c"]
        args:
        - |
          echo "Reading from subPath mount..."
          echo "Current directory: \$(pwd)"
          echo "Contents of /workspace:"
          ls -la /workspace/
          if [ -f /workspace/test.txt ]; then
            echo "Found test.txt! Contents:"
            cat /workspace/test.txt
          else
            echo "ERROR: test.txt not found in subPath mount!"
          fi
        volumeMounts:
        - name: workspace
          mountPath: /workspace
          subPath: ${SERVICE_NAME}
        workingDir: /workspace
      volumes:
      - name: workspace
        persistentVolumeClaim:
          claimName: shared-workspace
      nodeSelector:
        kubernetes.io/hostname: talos-a43-ee1
EOF

echo "Waiting for read job to complete..."
kubectl wait --for=condition=complete job/test-read-subpath -n $NAMESPACE --timeout=60s

echo ""
echo "Read job logs:"
kubectl logs job/test-read-subpath -n $NAMESPACE

# Cleanup
kubectl delete job test-write-full-path test-read-subpath -n $NAMESPACE

echo ""
echo "Test complete!"