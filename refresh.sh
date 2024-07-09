#!/bin/bash
# From: https://github.com/hedronvision/bazel-compile-commands-extractor

if ! [ -e external ] ; then
    ln -s bazel-out/../../../external .
fi

bazel run @hedron_compile_commands//:refresh_all

