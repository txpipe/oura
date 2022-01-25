# Cardano => Webhook Testdrive

## Introduction

This is a reference implementation to show how _Oura_ can be leveradged to read from a Cardano relay node into an output events via HTTP to a remote endpoint.

## Prerequisites

- K8s Cluster
- kubectl
- Skaffold

## Deployment

Create a k8s namespace for the testdrive:

```
kubectl create namespace cardano2webhook
```

Deploy the resources:

```
skaffold run --namespace cardano2webhook --tail
```

Check the logs:

```
skaffold run --module oura --namespace cardano2webhook
```