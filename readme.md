# openapi-mocker

Create a mock server from an OpenAPI 3.0 specification.

## Installation

- Install with cargo:

```bash
cargo install openapi-mocker
```

## Usage

1. Create an OpenAPI 3.0 specification file. For example, `openapi.yaml`:

    ```yaml
    openapi: 3.0.0
    info:
      title: Example API
      version: 1.0.0
    paths:
        /hello:
            get:
                responses:
                    '200':
                        description: OK
                        content:
                            application/json:
                                schema:
                                    type: object
                                    properties:
                                        message:
                                            type: string
                                    example:
                                        message: Hello, world!
                    '400':
                        description: Bad Request
                        content:
                            application/json:
                                schema:
                                    type: object
                                    properties:
                                        message:
                                            type: string
                                    example:
                                        message: Bad request
    ```

    > Note: The `example` field under the `content.schema` object is used
    to generate the mock response.

2. Run the mock server:

    ```bash
    openapi-mocker openapi.yaml
    ```

3. Perform an http request to the mock server:

    ```bash
    curl -i http://localhost:8080/hello
    ```

    The response should be:
        ```json
        {"message":"Hello, world!"}
        ```

    Requesting non existent route

    ```bash
    curl -i http://localhost:8080/does-not-exist
    ```

    The response should be 404 Not Found.

## Options

- `--port` or `-p`: Port to run the server on. Default is `8080`.

## Performing requests

The mock server will respond to any request with a response defined in the OpenAPI specification.
If the request does not match any of the paths defined in the specification, the server will respond with a 404 Not Found.

By default, requesting an existing path defined in the specification will return a 200 response
with the example response defined in the specification.

### Request different status codes

To request a different status code, use the base url with the status code as a path parameter.

For example, to request a 400 response example:

```bash
curl -i http://localhost:8080/400/hello
```

The response should be:

```json
{"message":"Bad request"}
```

> Note: The status code must be defined in the OpenAPI specification.
> Requesting a status code that is not defined in the specification will return a 404 Not Found.

## License

MIT
