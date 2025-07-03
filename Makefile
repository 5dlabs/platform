# Claude Code Telemetry Stack Makefile

# Docker image settings
REGISTRY ?= ghcr.io
REPO_OWNER ?= $(shell git config --get remote.origin.url | sed 's/.*github.com[:/]\([^/]*\).*/\1/')
REPO_NAME ?= $(shell basename `git rev-parse --show-toplevel`)
IMAGE_NAME := $(REGISTRY)/$(REPO_OWNER)/$(REPO_NAME)/claude-code
IMAGE_TAG ?= latest

.PHONY: help
help: ## Show this help message
	@echo 'Usage: make [target]'
	@echo ''
	@echo 'Targets:'
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  %-30s %s\n", $$1, $$2}' $(MAKEFILE_LIST)

# Variables
CLUSTER_NAME := kind-kind
NAMESPACE := telemetry
HELM_TIMEOUT := 10m
KIND_CONFIG := infrastructure/kind-config.yaml

# Cluster Management
.PHONY: cluster-up
cluster-up: ## Create Kind cluster (already exists, this recreates)
	kind create cluster --config $(KIND_CONFIG) --name $(CLUSTER_NAME)

.PHONY: cluster-down
cluster-down: ## Delete Kind cluster
	kind delete cluster --name $(CLUSTER_NAME)

.PHONY: cluster-info
cluster-info: ## Show cluster information
	kubectl cluster-info --context kind-$(CLUSTER_NAME)
	@echo "\nNodes:"
	kubectl get nodes
	@echo "\nNamespaces:"
	kubectl get namespaces

# Namespace Management
.PHONY: namespace-create
namespace-create: ## Create telemetry namespace
	kubectl create namespace $(NAMESPACE) --dry-run=client -o yaml | kubectl apply -f -
	kubectl label namespace $(NAMESPACE) name=$(NAMESPACE) --overwrite

# Helm Repository Management
.PHONY: helm-repos-add
helm-repos-add: ## Add required Helm repositories
	helm repo add victoria-metrics https://victoriametrics.github.io/helm-charts/
	helm repo add open-telemetry https://open-telemetry.github.io/opentelemetry-helm-charts
	helm repo add grafana https://grafana.github.io/helm-charts
	helm repo add ingress-nginx https://kubernetes.github.io/ingress-nginx
	helm repo update

# Component Deployment
.PHONY: deploy-nginx
deploy-nginx: namespace-create ## Deploy NGINX Ingress Controller
	helm upgrade --install ingress-nginx ingress-nginx/ingress-nginx \
		--namespace ingress-nginx \
		--create-namespace \
		--set controller.service.type=NodePort \
		--set controller.hostPort.enabled=true \
		--wait

.PHONY: deploy-otel
deploy-otel: namespace-create ## Deploy OpenTelemetry Collector
	helm upgrade --install otel-collector open-telemetry/opentelemetry-collector \
		--namespace $(NAMESPACE) \
		--values helm/values/otel-collector.yaml \
		--timeout $(HELM_TIMEOUT) \
		--wait

.PHONY: deploy-victoria-metrics
deploy-victoria-metrics: namespace-create ## Deploy VictoriaMetrics
	helm upgrade --install victoria-metrics victoria-metrics/victoria-metrics-single \
		--namespace $(NAMESPACE) \
		--values helm/values/victoria-metrics.yaml \
		--timeout $(HELM_TIMEOUT) \
		--wait

.PHONY: deploy-victoria-logs
deploy-victoria-logs: namespace-create ## Deploy VictoriaLogs
	helm upgrade --install victoria-logs victoria-metrics/victoria-logs-single \
		--namespace $(NAMESPACE) \
		--values helm/values/victoria-logs.yaml \
		--timeout $(HELM_TIMEOUT) \
		--wait

.PHONY: deploy-grafana
deploy-grafana: namespace-create ## Deploy Grafana
	helm upgrade --install grafana grafana/grafana \
		--namespace $(NAMESPACE) \
		--values helm/values/grafana.yaml \
		--timeout $(HELM_TIMEOUT) \
		--wait

.PHONY: deploy-claude-code
deploy-claude-code: namespace-create ## Deploy Claude Code with telemetry
	@if [ -z "$(ANTHROPIC_API_KEY)" ]; then \
		echo "Error: ANTHROPIC_API_KEY environment variable is required"; \
		echo "Usage: make deploy-claude-code ANTHROPIC_API_KEY=your-api-key"; \
		exit 1; \
	fi
	kubectl create namespace claude-code --dry-run=client -o yaml | kubectl apply -f -
	helm upgrade --install claude-code ./helm/claude-code \
		--namespace claude-code \
		--set secrets.anthropicApiKey=$(ANTHROPIC_API_KEY) \
		--set telemetry.githubUser=$${GITHUB_USER:-$(USER)} \
		--set telemetry.workingService=$${WORKING_SERVICE:-platform} \
		--timeout $(HELM_TIMEOUT) \
		--wait

.PHONY: deploy-claude-code-dev
deploy-claude-code-dev: namespace-create ## Deploy Claude Code in development mode (stdout + file)
	@if [ -z "$(ANTHROPIC_API_KEY)" ]; then \
		echo "Error: ANTHROPIC_API_KEY environment variable is required"; \
		echo "Usage: make deploy-claude-code-dev ANTHROPIC_API_KEY=your-api-key"; \
		exit 1; \
	fi
	kubectl create namespace claude-code-dev --dry-run=client -o yaml | kubectl apply -f -
	helm upgrade --install claude-code-dev ./helm/claude-code \
		--namespace claude-code-dev \
		--set developmentMode.enabled=true \
		--set secrets.anthropicApiKey=$(ANTHROPIC_API_KEY) \
		--set telemetry.githubUser=$${GITHUB_USER:-$(USER)} \
		--set telemetry.workingService=$${WORKING_SERVICE:-platform} \
		--timeout $(HELM_TIMEOUT) \
		--wait

.PHONY: test-claude-code-chart
test-claude-code-chart: ## Test Claude Code Helm chart
	helm lint ./helm/claude-code
	helm template claude-code ./helm/claude-code \
		--namespace claude-code \
		--set secrets.anthropicApiKey=test-key \
		--debug

# Alerting Configuration
.PHONY: deploy-alerting-rules
deploy-alerting-rules: namespace-create ## Deploy alerting rules ConfigMap
	@echo "Deploying alerting rules ConfigMap..."
	kubectl apply -f helm/alerts/alerting-rules-configmap.yaml

.PHONY: configure-grafana-alerts
configure-grafana-alerts: ## Configure alerts in Grafana via API
	@echo "Configuring Grafana alerts..."
	@scripts/configure-grafana-alerts.sh

.PHONY: setup-alerting
setup-alerting: deploy-alerting-rules configure-grafana-alerts ## Complete alerting setup

# Stack Management
.PHONY: deploy-all
deploy-all: helm-repos-add deploy-nginx deploy-otel deploy-victoria-metrics deploy-victoria-logs deploy-grafana ## Deploy entire telemetry stack
	@echo "Telemetry stack deployment complete!"
	@echo "Run 'make stack-status' to check component status"

.PHONY: stack-status
stack-status: ## Check status of all components
	@echo "=== Telemetry Stack Status ==="
	@echo "\nPods in $(NAMESPACE) namespace:"
	kubectl get pods -n $(NAMESPACE)
	@echo "\nServices in $(NAMESPACE) namespace:"
	kubectl get svc -n $(NAMESPACE)
	@echo "\nIngress resources:"
	kubectl get ingress -n $(NAMESPACE)

.PHONY: stack-remove
stack-remove: ## Remove entire telemetry stack
	helm uninstall grafana -n $(NAMESPACE) || true
	helm uninstall victoria-logs -n $(NAMESPACE) || true
	helm uninstall victoria-metrics -n $(NAMESPACE) || true
	helm uninstall otel-collector -n $(NAMESPACE) || true
	helm uninstall ingress-nginx -n ingress-nginx || true
	kubectl delete namespace $(NAMESPACE) || true

# Port Forwarding
.PHONY: port-forward-grafana
port-forward-grafana: ## Port forward Grafana (http://localhost:3000)
	@echo "Grafana will be available at http://localhost:3000"
	@echo "Default credentials: admin/admin (check helm values for actual password)"
	kubectl port-forward -n $(NAMESPACE) svc/grafana 3000:80

.PHONY: port-forward-vm
port-forward-vm: ## Port forward VictoriaMetrics (http://localhost:8428)
	@echo "VictoriaMetrics will be available at http://localhost:8428"
	kubectl port-forward -n $(NAMESPACE) svc/victoria-metrics-single-server 8428:8428

.PHONY: port-forward-otel
port-forward-otel: ## Port forward OTLP endpoints
	@echo "OTLP gRPC will be available at localhost:4317"
	@echo "OTLP HTTP will be available at localhost:4318"
	kubectl port-forward -n $(NAMESPACE) svc/otel-collector 4317:4317 4318:4318

# Testing
.PHONY: test-metrics-ingestion
test-metrics-ingestion: ## Test metric ingestion via OTLP
	@echo "Testing OTLP metric ingestion..."
	curl -X POST http://localhost:4318/v1/metrics \
		-H "Content-Type: application/json" \
		-d @test-data/sample-metrics.json || echo "Note: Create test-data/sample-metrics.json first"

.PHONY: test-logs-ingestion
test-logs-ingestion: ## Test log ingestion via OTLP
	@echo "Testing OTLP log ingestion..."
	curl -X POST http://localhost:4318/v1/logs \
		-H "Content-Type: application/json" \
		-d @test-data/sample-logs.json || echo "Note: Create test-data/sample-logs.json first"

.PHONY: test-comprehensive
test-comprehensive: ## Run comprehensive test suite for entire stack
	@echo "Running comprehensive test suite..."
	@./scripts/comprehensive-test.sh

.PHONY: test-quick
test-quick: ## Quick health check of all components
	@echo "Running quick health check..."
	@kubectl get pods -n telemetry | grep -E "(Running|Completed)" | wc -l | xargs echo "Healthy pods:"
	@kubectl get pods -n telemetry | grep -vE "(Running|Completed)" | grep -v "NAME" | wc -l | xargs echo "Unhealthy pods:"

# Utilities
.PHONY: get-grafana-password
get-grafana-password: ## Get Grafana admin password
	@echo "Grafana admin password:"
	@kubectl get secret --namespace $(NAMESPACE) grafana -o jsonpath="{.data.admin-password}" | base64 --decode
	@echo ""

.PHONY: logs
logs: ## Tail logs for all telemetry components
	kubectl logs -n $(NAMESPACE) -f --tail=100 --selector="app.kubernetes.io/part-of=telemetry"

.PHONY: clean
clean: ## Clean up temporary files
	rm -rf charts/
	rm -rf *.tgz
	find . -name "*.bak" -delete

# Docker targets
.PHONY: docker-build
docker-build: ## Build Claude Code Docker image locally
	@echo "Building Docker image: $(IMAGE_NAME):$(IMAGE_TAG)"
	docker build -t $(IMAGE_NAME):$(IMAGE_TAG) \
		--build-arg TZ=UTC \
		.

.PHONY: docker-push
docker-push: ## Push Docker image to registry (requires login)
	@echo "Pushing Docker image: $(IMAGE_NAME):$(IMAGE_TAG)"
	docker push $(IMAGE_NAME):$(IMAGE_TAG)

.PHONY: docker-run
docker-run: ## Run Claude Code container locally
	docker run -it --rm \
		-v $(PWD):/workspace \
		-v claude-sessions:/home/node/.claude \
		-e ANTHROPIC_API_KEY=$(ANTHROPIC_API_KEY) \
		$(IMAGE_NAME):$(IMAGE_TAG) \
		zsh

.PHONY: docker-shell
docker-shell: ## Get a shell in the Claude Code container
	docker run -it --rm \
		-v $(PWD):/workspace \
		-v claude-sessions:/home/node/.claude \
		--entrypoint /bin/zsh \
		$(IMAGE_NAME):$(IMAGE_TAG)