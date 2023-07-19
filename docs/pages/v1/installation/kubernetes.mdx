# Kubernetes

_Oura_ running in `daemon` mode can be installed can be deployed in Kubernetes cluster, Depending on your needs, we recommend any of the following approaches.

## Approach #1: Sidecar Container

_Oura_ can be loaded as a sidecar container in the same pod as your Cardano node. Since containers in a pod share the same file-system layer, it's easy to point _Oura_ to the unix-socket of the node.

In the following example yaml, we show a redacted version of a Cardano node resource defined a s Kubernetes STS. Pay attention on the extra container and to the volume mounts to share the unix socket.

```yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: cardano-node
spec:

  # REDACTED: here goes your normal cardano node sts / deployment spec

  template:
    spec:

      # REDACTED: here goes your normal cardano node pod specs

      containers:
      - name: main

        # REDACTED: here goes your normal cardano node container properties

        # add a new volume mount to enable the socket to be
        # consumed by the 2nd container in the pod (Oura)
        volumeMounts:
        - mountPath: /opt/cardano/cnode/sockets/node0.socket
          name: unix-socket

      # add a 2nd container pointing to the _Oura_ image
      - name: oura
        image: ghcr.io/txpipe/oura:latest

        # we mount the same volume that the main container uses as the source
        # for the Cardano node unix socket.
        volumeMounts:
        - mountPath: /opt/cardano/cnode/sockets/node0.socket
          name: unix-socket
        - mountPath: /etc/oura
          name: oura-config

      volumes:

      # REDACTED: here goes any required volume for you normal cardano node setup

      # empty-dir volume to share the unix socket between containers
      - name: unix-socket
        emptyDir: {}

      # a config map resource with Oura's config particular for your requirements
      - name: oura-config
        configMap:
          name: oura-config
```


## Approach #2: Standalone Deployment

_Oura_ can be implemented as a standalone Kubernetes `deployment` resource. This is useful if your Cardano node is not part of your Kubernetes cluster or if you want to keep your node strictly isolated from the access of a sidecard pod.

Please note that the amount of replicas is set to `1`. _Oura_ doesn't have any kind of "coordination" between instances. Adding more than one replica will just create extra pipelines duplicating the same work.

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: oura
data:
  daemon.toml: |-
    [source]
    # REDACTED: here goes your `source` configuration options

    [[filters]]
    # REDACTED: here goes your `filters` configuration options

    [sink]
    # REDACTED: here goes your `sink` configuration options
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: oura
  labels:
    app: oura
spec:
  replicas: 1
  selector:
    matchLabels:
      app: oura
  template:
    metadata:
      labels:
        app: oura
    spec:
      containers:
      - name: main
        image: ghcr.io/txpipe/oura:latest
        env:
          - name: "RUST_LOG"
            value: "info"
        resources:
          requests:
            memory: 50Mi
            cpu: 50m
          limits:
            memory: 200Mi
            cpu: 200m
        args:
          - "daemon"
        volumeMounts:
          - mountPath: /etc/oura
            name: config
      volumes:
      - name: config
        configMap:
          name: oura
```
