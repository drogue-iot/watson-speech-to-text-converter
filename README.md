# IBM Watson speech-to-text converter for Knative

[![CI](https://github.com/drogue-iot/watson-speech-to-text-converter/workflows/CI/badge.svg)](https://github.com/drogue-iot/watson-speech-to-text-converter/actions?query=workflow%3A%22CI%22)
[![GitHub release (latest SemVer)](https://img.shields.io/github/v/tag/drogue-iot/watson-speech-to-text-converter?sort=semver)](https://github.com/orgs/drogue-iot/packages/container/package/watson-speech-to-text-converter)
[![Matrix](https://img.shields.io/matrix/drogue-iot:matrix.org)](https://matrix.to/#/#drogue-iot:matrix.org)

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

## Payload

The payload must be an accepted audio format (with a matching mime type in the *data content type*).
You can find more information about that in the section [Audio Formats](https://cloud.ibm.com/docs/speech-to-text?topic=speech-to-text-audio-formats)
of IBM Watson.

Of course, there is a special case. There are two different mime types for WAV
files (`audio/wav` and `audio/vnd.wav`). Watson accepts only `audio/wav`. If you have `audio/vnd.wav`, you need to
convert that first. However, the converter can do that for you if `FIX_WAV_TYPE` is set to `true` (which is the
default).

So if your content type is e.g. `audio/vnd.wav; codec=1`, it will automatically translated into `audio/wav`
unless you explicitly disable the logic by setting `FIX_WAV_TYPE=false`.

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
| `FIX_WAV_TYPE` | `true` | Enable fixing of WAV content type (see section [#payload](Payload)) | 

## Filter

A filter can be applied to incoming events. Events that do not match the filter criteria are discarded. In a nutshell,
the filter requires all criteria to pass, an empty filter configuration automatically passes.
