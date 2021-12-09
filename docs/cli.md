# CLI Instructions

## Installation

### MacOS

To install Oura binary in a Mac, use the following shell command:

```sh
curl -L -o oura.tar.gz https://github.com/txpipe/oura/releases/latest/download/oura-x86_64-apple-darwin.tar.gz && tar xvzf oura.tar.gz && mv oura /usr/local/bin
```

### Ubuntu

To install Oura binary in a Mac, use the following shell command:

```sh
curl -L -o oura.tar.gz https://github.com/txpipe/oura/releases/latest/download/oura-x86_64-unknown-linux-gnu.tar.gz && tar xvzf oura.tar.gz && mv oura /usr/local/bin
```

Check the [latest release](https://github.com/txpipe/oura/releases/latest) for more binary download options.

## Usage

### Watch Live Data from Remote Relay Node

```sh
oura watch relays-new.cardano-mainnet.iohk.io:3001 --bearer tcp
```

### Watch Live Data from a Local Node via Unix Socket

```sh
oura watch /opt/cardano/cnode/sockets/node0.socket --bearer unix
```