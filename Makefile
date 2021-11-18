all: engine

VACHCLI = vach-cli
KEYFILE = ./res/keys/key.pair
PNG_ASSET_PAK = ./res/redist/assets.pak
SHADER_ASSET_PAK = ./res/redist/shaders.pak

engine: Makefile
	cargo build

assets: png_assets shader_assets

png_assets: res/assets/*.png
	$(VACHCLI) -K $(KEYFILE) package -E $(PNG_ASSET_PAK) ./res/assets/*.png

shader_assets: res/shaders/*.wgsl
	cargo wgsl
	$(VACHCLI) -K $(KEYFILE) package -E $(SHADER_ASSET_PAK) ./res/shaders/*.wgsl
