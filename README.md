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
