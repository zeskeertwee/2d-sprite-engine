all: engine

VACHCLI = vach
KEYFILE = ../keys/key.pair
PNG_ASSET_PAK = ../redist/assets.pak
SHADER_ASSET_PAK = ../redist/shaders.pak
SCRIPT_ASSET_PAK = ../redist/scripts.pak

engine: Makefile
	cargo build

assets: clean png_assets shader_assets script_assets

clean:
	rm ./res/redist/*

png_assets: res/assets/*.png
	cd ./res/assets && $(VACHCLI) pack -k $(KEYFILE) -e -a -o $(PNG_ASSET_PAK) -i *.png

shader_assets: res/shaders/*.wgsl
	cargo wgsl
	cd ./res/shaders && $(VACHCLI) pack -k $(KEYFILE) -e -a -o $(SHADER_ASSET_PAK) -i *.wgsl

script_assets: res/assets/*.wasm
	cd ./res/assets && $(VACHCLI) pack -k $(KEYFILE) -e -a -o $(SCRIPT_ASSET_PAK) -i *.wasm