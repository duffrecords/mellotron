#!/bin/bash

if [ "$1" = "debug" ]; then
    mode=""
else
    mode="--release"
fi
plugin_name="mellotron"
cargo build $mode || exit 1
rc=$?
[ $rc -ne 0 ] && exit rc
mkdir -p ~/.lv2/${plugin_name}.lv2
cp -a lv2/*.ttl ~/.lv2/${plugin_name}.lv2/
cp -a target/release/lib${plugin_name}.so ~/.lv2/${plugin_name}.lv2/
cp -a target/release/lib${plugin_name}_ui.so ~/.lv2/${plugin_name}.lv2/
mkdir -p ~/.lv2/${plugin_name}.lv2/samples
rsync -ar samples/ ~/.lv2/${plugin_name}.lv2/samples/
