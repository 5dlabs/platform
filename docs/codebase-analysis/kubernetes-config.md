# kubernetes-config Analysis

**Path:** `infra/cluster-config`
**Type:** KubernetesConfig
**Lines of Code:** 92
**Description:** kubernetes-config configuration and files

## Source Files

### otel-collector-metrics-service.yaml (18 lines)

**Full Content:**
```yaml
apiVersion: v1
kind: Service
metadata:
  name: otel-collector-metrics
  namespace: telemetry
  labels:
    app.kubernetes.io/name: opentelemetry-collector
    app.kubernetes.io/instance: otel-collector
spec:
  type: ClusterIP
  ports:
  - name: metrics
    port: 8890
    targetPort: 8890
    protocol: TCP
  selector:
    app.kubernetes.io/name: opentelemetry-collector
    app.kubernetes.io/instance: otel-collector
```

### otel-prometheus-service.yaml (22 lines)

**Full Content:**
```yaml
apiVersion: v1
kind: Service
metadata:
  name: otel-collector-metrics
  namespace: telemetry
  labels:
    app.kubernetes.io/name: opentelemetry-collector
    app.kubernetes.io/instance: otel-collector
spec:
  type: ClusterIP
  ports:
    - name: prometheus
      port: 8889
      targetPort: 8889
      protocol: TCP
    - name: internal-metrics
      port: 8890
      targetPort: 8890
      protocol: TCP
  selector:
    app.kubernetes.io/name: opentelemetry-collector
    app.kubernetes.io/instance: otel-collector
```

### local-path-config-patch.yaml (44 lines)

**Full Content:**
```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: local-path-config
  namespace: local-path-storage
data:
  config.json: |-
    {
            "nodePathMap":[
            {
                    "node":"DEFAULT_PATH_FOR_NON_LISTED_NODES",
                    "paths":["/var/mnt/local-path-provisioner"]
            }
            ]
    }
  helperPod.yaml: |-
    apiVersion: v1
    kind: Pod
    metadata:
      name: helper-pod
    spec:
      priorityClassName: system-node-critical
      tolerations:
        - key: node.kubernetes.io/disk-pressure
          operator: Exists
          effect: NoSchedule
      securityContext:
        fsGroup: 0
        runAsUser: 0
        runAsGroup: 0
      containers:
      - name: helper-pod
        image: busybox
        imagePullPolicy: IfNotPresent
        securityContext:
          privileged: true
  setup: |-
    #!/bin/sh
    set -eu
    mkdir -m 0777 -p "$VOL_DIR"
  teardown: |-
    #!/bin/sh
    set -eu
    rm -rf "$VOL_DIR"
```

### talos-local-path-volume.yaml (8 lines)

**Full Content:**
```yaml
apiVersion: v1alpha1
kind: UserVolumeConfig
name: local-path-provisioner
provisioning:
  diskSelector:
    match: "!system_disk"
  minSize: 100GB
  maxSize: 100GB
```

