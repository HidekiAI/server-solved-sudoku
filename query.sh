#!/bin/bash
bazel query --notool_deps --noimplicit_deps "deps(//protobuf:route_proto)"
