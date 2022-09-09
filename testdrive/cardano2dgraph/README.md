# Cardano => dgraph Testdrive

## Introduction

This is a reference implementation to show how _Oura_ can be leveradged to read from a Cardano relay node into an output events via HTTP to a remote endpoint, in this case a dgraph instance passing through a payload transformer that makes oura events fit dgraph expected payload.

Note the payload transformer [(comcast/eel)](https://github.com/Comcast/eel) runs multiple replicas (load balanced using haproxy) to achieve higher throughput, and although it manages its own buffer and handle failed requests to dgraph and tries to resubmit them, events can be lost in case any of these replicas are unexpectedly restarted.

## Prerequisites

- K8s Cluster
- kubectl
- Skaffold
- kustomize

## Deployment

Create a k8s namespace for the testdrive:

```
kubectl create namespace cardano2dgraph
```

Deploy the resources:

```
skaffold run --namespace cardano2dgraph
```

Check events going thru:
```
kubectl logs -n cardano2dgraph -f -l app=oura-2-dgraph-etl
```

## Access dgraph

Expose `dgraph` api (will sit in http://localhost:8081):
```
kubectl port-forward -n cardano2dgraph svc/dgraph-public 8081:8080
```

Optionally, use [dgraph/ratel](https://github.com/dgraph-io/ratel) webui to access `dgraph` (access http://localhost:8001 and use http://localhost:8081 as dgraph endpoint):
```
docker run --rm -d -p 8001:8000 dgraph/ratel:latest
```