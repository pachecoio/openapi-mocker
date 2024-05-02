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
                                examples:
                                    default:
                                        value:
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
                                examples:
                                    default:
                                        value:
                                            message: Bad request
    ```

2. Run the mock server:

    ```bash
    openapi-mocker openapi.yaml
    ```

    - Or, Run with docker:

    ```bash
    docker run \
        -v $(pwd)/tests/testdata/petstore.yaml:/openapi.yaml \
        -p 8080:8080 \
        thisk8brd/openapi-mocker:latest \
        /openapi.yaml
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

You can use custom examples defined in the OpenAPI specification to test different responses.
Custom examples can be defined and requested in different ways.

## Requesting by path

You can define an example with the exact path you want to match.
Example:
    
    ```yaml
    openapi: 3.0.0
    info:
      title: Example API
      version: 1.0.0
    paths:
        /hello/{name}:
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
                                examples:
                                    default:
                                        value:
                                            message: Hello, world!
                                    /hello/jon_snow:
                                        value:
                                            message: You know nothing, Jon Snow!
    ```

Request the example by the exact path:
    ```bash
    curl -i http://localhost:8080/hello/jon_snow
    ```
    
    The response should be:
    
    ```json
    {"message":"You know nothing, Jon Snow!"}
    ```

Request the default example:
    ```bash
    curl -i http://localhost:8080/hello/arya_stark
    ```
    
    The response should be:
    
    ```json
    {"message":"Hello, world!"}
    ```

## Requesting by query parameter

You can define an example with a query parameter you want to match.

Example:
    
    ```yaml
    openapi: 3.0.0
    info:
      title: Example API
      version: 1.0.0
    paths:
        /hello:
            get:
                parameters:
                    - name: name
                      in: query
                      required: true
                      schema:
                        type: string
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
                                examples:
                                    default:
                                        value:
                                            message: Hello, world!
                                    "query:name=sansa":
                                        value:
                                            message: Sansa Stark
    ```

Request the example by the query parameter:
    ```bash
    curl -i http://localhost:8080/hello?name=sansa
    ```
    The response should be:
    ```json
    {"message": "Sansa Stark"}
    ```

Request that does not match the query parameter:
    ```bash
    curl -i http://localhost:8080/hello?name=arya
    ```
    The response should be:
    ```json
    {"message": "Hello, world!"}
    ```

## Requesting by headers

You can define an example with a header you want to match.

Example:
    
    ```yaml
    openapi: 3.0.0
    info:
      title: Example API
      version: 1.0.0
    paths:
        /hello:
            get:
                parameters:
                    - name: name
                      in: header
                      required: true
                      schema:
                        type: string
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
                                examples:
                                    default:
                                        value:
                                            message: Hello, world!
                                    "header:x-name=tyrion":
                                        value:
                                            message: Tyrion Lannister
    ```

Request the example by the header:
    ```bash
    curl -i http://localhost:8080/hello -H "x-name: tyrion"
    ```
    The response should be:
    ```json
    {"message": "Tyrion Lannister"}
    ```
    
Request that does not match the header:
    ```bash
    curl -i http://localhost:8080/hello
    ```
    The response should be:
    ```json
    {"message": "Hello, world!"}
    ```

> Note: The matches occur in the following order: path, query, headers.
> It is also important to note that the request is going to return the 
> first match found in the order above. If no match is found, the default
> example is going to be returned.

> Note: The matches are applied accross all the examples and responses in the OpenAPI specification.

## Contributing

Contributions are welcome! Please see the [contributing guidelines](CONTRIBUTING.md).

## License

MIT
