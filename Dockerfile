FROM rust:1-alpine3.19
ENV RUSTFLAGS="-C target-feature=-crt-static"
RUN apk add --no-cache musl-dev 
WORKDIR /app
COPY ./ /app
RUN cargo build --release 
RUN strip target/release/openapi-mocker 

FROM alpine:3.19
RUN apk add --no-cache libgcc
COPY --from=0 /app/target/release/openapi-mocker .
EXPOSE 8080
ENTRYPOINT ["/openapi-mocker"]
