# Deploy

Create a `credentials.json` with the access credentials from the portal. Then execute:

    oc create secret generic watson-sst-credentials --from-file=deploy/credentials.json
    oc apply -f deploy/020-KnativeService.yaml
