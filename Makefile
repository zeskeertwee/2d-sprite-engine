all: engine

ROOT_DIR:=$(shell dirname $(realpath $(firstword $(MAKEFILE_LIST))))
VACHCLI = vach
KEYFILE = $(ROOT_DIR)/res/keys/key.pair
PNG_ASSET_PAK = $(ROOT_DIR)/res/redist/assets.pak
SHADER_ASSET_PAK = $(ROOT_DIR)/res/redist/shaders.pak
SCRIPT_ASSET_PAK = $(ROOT_DIR)/res/redist/scripts.pak
LVME_ASSET_PAK = $(ROOT_DIR)/res/redist/lvme.pak

engine: Makefile
	cargo build

assets: clean png_assets shader_assets script_assets lvme

clean:
	rm ./res/redist/*

lvme: engine/lvme-impl/*.lua
	cd ./engine/lvme-impl && $(VACHCLI) pack -k $(KEYFILE) -e -a -o $(LVME_ASSET_PAK) -i *.lua

png_assets: res/assets/*.png
	cd ./res/assets && $(VACHCLI) pack -k $(KEYFILE) -e -a -o $(PNG_ASSET_PAK) -i *.png

shader_assets: res/shaders/*.wgsl
	cargo wgsl
	cd ./res/shaders && $(VACHCLI) pack -k $(KEYFILE) -e -a -o $(SHADER_ASSET_PAK) -i *.wgsl

script_assets: res/assets/*.lua
	cd ./res/assets && $(VACHCLI) pack -k $(KEYFILE) -e -a -o $(SCRIPT_ASSET_PAK) -i *.lua
