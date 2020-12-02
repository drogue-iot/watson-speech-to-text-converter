# IBM Watson speech-to-text converter for Knative

Converts audio payload to text, using *IBM Watson Speech to Text*.

## Input

Cloud event:

* **Data Content Type**: Mime type of the audio payload (e.g. `audio/wav`)
* **Payload**: Binary audio payload in the type specified by *data content type* 

## Output

The output event will contain all information of the input event, with the following changes:

* **Data Content Type**: `application/json`
* **Data Schema**: `https://cloud.ibm.com/services/speech-to-text/v1/recognize`
* **Payload**: Response JSON, with recognized text.

## Installation

See [deploy/](deploy/) for information how to install.

## Configuration

You can use the following environment variables to configure its behavior:

| Name | Default | Description |
| ---- | ------- | ----------- |
| `BIND_ADDR` | `127.0.0.1:8080` | The address the HTTP server binds to |
| `RUST_LOG` | none | The configuration of the logger, also see https://docs.rs/env_logger/latest/env_logger/ |
| `CREDENTIALS_PATH` | `/etc/config/credentials.json` | The path to the credential file |
| `ONLY_TYPES` | none | A comma separated list of allowed *types*. If the list is empty the filter will not be applied and the event will pass. |
| `ONLY_SUBJECTS` | none |  A comma separated list of allowed *subjects*. If the list is empty the filter will not be applied and the event will pass. |
| `ONLY_DATACONTENTTYPES` | none |  A comma separated list of allowed *data content types*. If the list is empty the filter will not be applied and the event will pass. |

## Filter

A filter can be applied to incoming events. Events that do not match the filter criteria are discarded. In a nutshell,
the filter requires all criteria to pass, an empty filter configuration automatically passes.
