# Deploy

Unfortunately, there is no *ready to run* deployment, as you will need to include the
*convert speech to text* step into some bigger example.

The service itself can be deployed as described show in [knative-service.yaml](knative-service.yaml). However, you
will also need to wire it up, so that events flow in and out of the service.

That deployment requires a secret, mapped into the container, which contains the access credentials for
the API. You can copy and paste the credentials from the Watson portal, and create a secret as show in the following
*example* section. 

## Example

This example shows a use of the container in an IoT related example for
[drogue-cloud](https://github.com/drogue-iot/drogue-cloud).

Create a `credentials.json` with the access credentials from the portal. Then execute:

    kubectl create secret generic watson-sst-credentials --from-file=deploy/credentials.json
    kubectl apply -k deploy
