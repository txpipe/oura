# Cardano => Elasticsearch Testdrive

## Introduction

This is a reference implementation to show how _Oura_ can be leveradged to build read from a Cardano relay node into an Elasticsearch cluster.

## Prerequisites

- K8s Cluster
- kubectl
- Skaffold

## Deployment

Install Elasticsearch official Kubernetes operator:

```
kubectl create -f https://download.elastic.co/downloads/eck/1.9.1/crds.yaml
kubectl apply -f https://download.elastic.co/downloads/eck/1.9.1/operator.yaml
```

Create a k8s namespace for the testdrive:

```
kubectl create namespace cardano2elastic
```

Deploy the Elasticsearch + Kibana resources:

```
skaffold run --module elastic --namespace cardano2elastic
```

Setup an index template to store _Oura_ events:

```
export ELASTIC_AUTH=user:pass
cd scripts && ./setup-index.sh
```

```
skaffold run --namespace cardano2elastic
```