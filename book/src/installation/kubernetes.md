# Kubernetes

_Oura_ running in `daemon` mode can be installed can be deployed in Kuberentes cluster, Depending on your needs, we recommend any of the following approaches.

## Approach #1: Sidecar Container

_Oura_ can loaded as a sidecar container in the same pod as your Cardano node. Since containers in a pod share the same file-system layer, it's easy to point _Oura_ to the unix-socket of the node.

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


## Approach #2: Independent Deployment

```
//TODO
```