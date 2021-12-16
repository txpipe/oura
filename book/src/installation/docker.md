# Docker

_Oura_ can be executed as a Docker container:

```sh
docker run ghcr.io/txpipe/oura:latest
```

The entry-point of the image points to _Oura_ executable. You can pass the same command-line arguments that you would pass to the binary release running bare-metal. For example:

```
docker run ghcr.io/txpipe/oura:latest \
    watch relays-new.cardano-mainnet.iohk.io:3001 \
    --bearer tcp
```

For more information on available command-line arguments, check the [usage](../usage/index.md) section.