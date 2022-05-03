# Build Oura by version

## Introduction

This is a reference implementation to show how an image of _Oura_ can be built on demand; but also, sending the git revision as an argument through the command line

## Prerequisites

- Docker

## Variables and arguments

|Build Argument|Type|Default|Required|What it is|
|---|---|---|---|---|
|`RELEASE_TAG`|ARG|Last workin version|no|Tag number of the oura release. Follow this [link](https://github.com/txpipe/oura/tags) for reference.|


### Build a Docker images with the latest Oura release
```bash
$ sudo docker build -t oura:latest .
```
```bash
$ sudo docker run --rm oura:latest --version
```
```bash
oura 1.3.2
```

### Build a Docker image of Oura v1.0.0
```bash
$ sudo docker build -t oura:v1.0.0 . \
--build-arg RELEASE_TAG=v1.0.0
```
```bash
$ sudo docker run --rm oura:v1.0.0.0 --version
```
```bash
oura 1.0.0
```