UNAME_S := $(shell uname -s)

OUTDIR := $(abspath _build)

ARCH ?= universal
WITH_GUI ?= 1

RUST_ARM64 := aarch64-apple-darwin
RUST_X86 := x86_64-apple-darwin

GUI_PROJECT := ventrica_mac/Ventrica.xcodeproj
GUI_SCHEME := Ventrica

BIN_DIR := $(OUTDIR)/bin
LIBEXEC_DIR := $(OUTDIR)/usr/libexec
PLIST_DIR := $(OUTDIR)/Library/LaunchDaemons

PLIST := layout/com.claration.ventricad.plist

ifeq ($(ARCH),arm64)
RUST_TARGETS := $(RUST_ARM64)
XCODE_ARCHS := arm64
endif

ifeq ($(ARCH),x86_64)
RUST_TARGETS := $(RUST_X86)
XCODE_ARCHS := x86_64
endif

ifeq ($(ARCH),universal)
RUST_TARGETS := $(RUST_ARM64) $(RUST_X86)
XCODE_ARCHS := arm64 x86_64
endif

ifeq ($(WITH_GUI),1)
GUI_TARGET := gui-build
else
GUI_TARGET :=
endif

.PHONY: all build clean rust xcframework gui-build package

all: build

build: clean rust xcframework $(GUI_TARGET) package

rust:
	@for t in $(RUST_TARGETS); do \
		echo "Building $$t"; \
		cargo build --release --target $$t; \
	done

xcframework:
ifeq ($(UNAME_S),Darwin)
	rm -rf ventrica_ffi/_build/libs
	mkdir -p ventrica_ffi/_build/libs
ifeq ($(ARCH),arm64)
	cp target/$(RUST_ARM64)/release/libventrica_ffi.a \
		ventrica_ffi/_build/libs/ventrica-macos.a
else ifeq ($(ARCH),x86_64)
	cp target/$(RUST_X86)/release/libventrica_ffi.a \
		ventrica_ffi/_build/libs/ventrica-macos.a
else
	lipo -create \
		target/$(RUST_ARM64)/release/libventrica_ffi.a \
		target/$(RUST_X86)/release/libventrica_ffi.a \
		-output ventrica_ffi/_build/libs/ventrica-macos.a

endif
	xcodebuild -create-xcframework \
		-library ventrica_ffi/_build/libs/ventrica-macos.a \
		-headers ventrica_ffi/_build/include \
		-output ventrica_ffi/_build/libVentrica.xcframework
else
	$(error XCFramework requires macOS)
endif

gui-build: xcframework
	xcodebuild \
		-project $(GUI_PROJECT) \
		-scheme $(GUI_SCHEME) \
		-configuration Release \
		ARCHS="$(XCODE_ARCHS)" \
		ONLY_ACTIVE_ARCH=NO \
		DEPLOYMENT_LOCATION=YES \
		DSTROOT=$(OUTDIR)

package:
	mkdir -p $(BIN_DIR) $(LIBEXEC_DIR) $(PLIST_DIR)
	cp $(PLIST) $(PLIST_DIR)/

ifeq ($(ARCH),arm64)
	cp target/$(RUST_ARM64)/release/ven $(BIN_DIR)/
	cp target/$(RUST_ARM64)/release/ventricad $(LIBEXEC_DIR)/

else ifeq ($(ARCH),x86_64)
	cp target/$(RUST_X86)/release/ven $(BIN_DIR)/
	cp target/$(RUST_X86)/release/ventricad $(LIBEXEC_DIR)/
else
	lipo -create \
		target/$(RUST_ARM64)/release/ven \
		target/$(RUST_X86)/release/ven \
		-output $(BIN_DIR)/ven
	lipo -create \
		target/$(RUST_ARM64)/release/ventricad \
		target/$(RUST_X86)/release/ventricad \
		-output $(LIBEXEC_DIR)/ventricad
endif

clean:
	rm -rf $(OUTDIR)
	rm -rf ventrica_ffi/_build/libs
	rm -rf ventrica_ffi/_build/libVentrica.xcframework