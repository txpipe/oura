# Watch Mode

## Watch Live Data from Remote Relay Node

```sh
oura watch relays-new.cardano-mainnet.iohk.io:3001 --bearer tcp
```

## Watch Live Data from a Local Node via Unix Socket

```sh
oura watch /opt/cardano/cnode/sockets/node0.socket --bearer unix
```