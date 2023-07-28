#!/bin/bash
set -e

export LOCAL_REGISTRY=localhost:5000
export TAG=$(date +%Y%m%d%H%M%S)
export IMAGE=$LOCAL_REGISTRY/hyperlane-cardano-local:$TAG

docker build -t $IMAGE .

docker push $IMAGE

echo "Pushed docker image $IMAGE to the local Docker registry $LOCAL_REGISTRY"