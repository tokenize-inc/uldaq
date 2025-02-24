FROM rust:1.82-slim

ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install --yes --no-install-recommends \
      gcc \
      g++ \
      libusb-1.0-0-dev \
      libclang1-14 \
    && \
    rm -rf /var/cache/apt
ENV CC=gcc \
    CXX=g++
