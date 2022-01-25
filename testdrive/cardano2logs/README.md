# Cardano => Logs Testdrive

## Introduction

This is a reference implementation to show how _Oura_ can be leveradged to read from a Cardano relay node into a set of rotating logs in the file system.

## Prerequisites

- K8s Cluster
- kubectl
- Skaffold

## Deployment

Create a k8s namespace for the testdrive:

```
kubectl create namespace cardano2logs
```

Deploy the resources:

```
skaffold run --namespace cardano2logs --tail
```