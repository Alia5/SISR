# SISR Makefile
# Cross-platform build automation for SISR

############################################################
# Variables
# These are defined in a cross-platform way. We branch early
# so that later variable definitions do not need per-OS logic.
############################################################

BINARY_NAME := sisr
MAIN_PKG := ./cmd/sisr
SRC_DIR := .
DIST_DIR := dist

# OS-specific helpers
ifeq ($(OS),Windows_NT)
	NULL_DEVICE := nul
	DATE_CMD := powershell -NoProfile -NonInteractive -Command "Get-Date -Format 'yyyy-MM-dd_HH:mm:ss'"
	EXE_EXT := .exe
	RM_DIR := rmdir /S /Q
	RM_FILE := del /Q
	COVERAGE_OUT := $(SRC_DIR)\coverage.out
	COVERAGE_HTML := $(SRC_DIR)\coverage.html
	export CGO_ENABLED=0
else
	NULL_DEVICE := /dev/null
	DATE_CMD := date -u +"%Y-%m-%d_%H:%M:%S"
	EXE_EXT :=
	RM_DIR := rm -rf
	RM_FILE := rm -f
	COVERAGE_OUT := $(SRC_DIR)/coverage.out
	COVERAGE_HTML := $(SRC_DIR)/coverage.html
	export CGO_ENABLED=0
endif

# Git-derived metadata (robust to missing git by redirecting errors)
VERSION ?= $(shell git describe --tags --match "v[0-9]*.[0-9]*.[0-9]*" --always 2>$(NULL_DEVICE) || echo v0.0.0-dev)
COMMIT := $(shell git rev-parse --short HEAD 2>$(NULL_DEVICE) || echo unknown)
BUILD_TIME := $(shell $(DATE_CMD))

# Go build flags
LDFLAGS := -s -w -X main.version=$(VERSION) -X main.commit=$(COMMIT) -X main.buildTime=$(BUILD_TIME) -X sisr/internal/codegen/common.Version=$(VERSION)
BUILD_FLAGS := -trimpath -ldflags "$(LDFLAGS)"

############################################################
# (Legacy) Windows detection section replaced by unified block
############################################################

.PHONY: all
all: test build

.PHONY: help
help: ## Show this help message
	@echo SISR Makefile
	@echo.
	@echo Usage: make [target]
	@echo.
	@echo Targets:
	@echo   help                 Show this help message
	@echo   deps                 Download Go dependencies
	@echo   tidy                 Tidy Go dependencies
	@echo   build                Build for current platform
	@echo   test                 Run tests
	@echo   test-coverage        Run tests with coverage
	@echo   clean                Remove build artifacts
	@echo   fmt                  Format Go code
	@echo   vet                  Run go vet
	@echo   lint                 Run golangci-lint
	@echo   run                  Build and run SISR
	@echo   run-server           Build and run SISR server
	@echo   docs-serve           Serve MkDocs documentation locally
	@echo   docs-build           Build MkDocs documentation
	@echo   docs-deploy          Deploy documentation to GitHub Pages
	@echo   version              Show version information

.PHONY: deps
deps: ## Download Go dependencies
	cd $(SRC_DIR) && go mod download

.PHONY: tidy
tidy: ## Tidy Go dependencies
	cd $(SRC_DIR) && go mod tidy

.PHONY: vet
vet: ## Run go vet
	cd $(SRC_DIR) && go vet ./...

.PHONY: test
test: ## Run tests
	cd $(SRC_DIR) && go test -count=1 -v ./...

.PHONY: test-coverage
test-coverage: ## Run tests with coverage
	cd $(SRC_DIR) && go test -count=1 -coverprofile=coverage.out ./...
	cd $(SRC_DIR) && go tool cover -html=coverage.out -o coverage.html

.PHONY: build
build: ## Build for current platform
	cd $(SRC_DIR) && go build $(BUILD_FLAGS) -o ./$(DIST_DIR)/$(BINARY_NAME)$(EXE_EXT) $(MAIN_PKG)

.PHONY: clean
clean: ## Remove build artifacts
	-@$(RM_DIR) $(DIST_DIR) 2>$(NULL_DEVICE)
	-@$(RM_FILE) $(COVERAGE_OUT) 2>$(NULL_DEVICE)
	-@$(RM_FILE) $(COVERAGE_HTML) 2>$(NULL_DEVICE)

.PHONY: fmt
fmt: ## Format Go code
	cd $(SRC_DIR) && go fmt ./...

.PHONY: lint
lint: ## Run golangci-lint (requires golangci-lint installed)
	cd $(SRC_DIR) && golangci-lint run

.PHONY: run
run: ## Build and run SISR
	cd $(SRC_DIR) && go run $(MAIN_PKG)

.PHONY: run-server
run-server: ## Build and run SISR server with default settings
	cd $(SRC_DIR) && go run $(MAIN_PKG) server

.PHONY: docs-serve
docs-serve: ## Serve MkDocs documentation locally (latest dev version)
	mike serve

.PHONY: docs-build
docs-build: ## Build MkDocs documentation
	mkdocs build

.PHONY: docs-deploy-dev
docs-deploy-dev: ## Deploy dev documentation version to GitHub Pages
	mike deploy --push --update-aliases dev latest

.PHONY: version
version: ## Show version information
	@echo Version: $(VERSION)
	@echo Commit:  $(COMMIT)
	@echo Built:   $(BUILD_TIME)
