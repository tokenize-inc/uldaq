#!/bin/bash

root=$(realpath "$(dirname "${0}")")

set -e
set -x

docker build \
  -t tokenizeinc.jfrog.io/default-docker/uldaq-builder \
  -f "${root}/Dockerfile" \
  "${root}"

docker run \
  -it \
  --rm \
  -v "${root}:${root}" \
  -w "${root}" \
  -u "$(id -u):$(id -g)" \
  tokenizeinc.jfrog.io/default-docker/uldaq-builder \
  "${@}"
