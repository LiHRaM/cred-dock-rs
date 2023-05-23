# cred-dock-rs

Based on a script from my blog: [https://www.hilmargustafs.com/posts/2023-05-22-document-your-problems/]

```bash
#! /usr/bin/env bash

set -euxo pipefail

ADC=~/.config/gcloud/application_default_credentials.json
ADC_DOCKER=/tmp/keys/creds.json
IMAGE_HASH=$(docker build -q .)

env -u DOCKER_DEFAULT_PLATFORM \
    docker run --rm \
        -e GOOGLE_APPLICATION_CREDENTIALS=${ADC_DOCKER} \
        -e GOOGLE_CLOUD_PROJECT=podimo-ai-prod \
        -v ${ADC}:${ADC_DOCKER}:ro \
        "${IMAGE_HASH}" \
        "$@"
```
