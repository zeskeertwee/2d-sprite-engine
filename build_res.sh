#!/bin/bash

./vach-cli -K ./res/keys/key.pair package -E ./res/redist/shaders.pak ./res/shaders/*.wgsl
./vach-cli -K ./res/keys/key.pair package -E ./res/redist/assets.pak ./res/assets/*.png
