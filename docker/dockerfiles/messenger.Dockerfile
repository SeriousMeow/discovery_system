FROM rust:1-alpine AS builder

WORKDIR /app/sources

RUN apk add --no-cache musl-dev pkgconfig openssl-dev build-base
RUN cargo install oas3-gen

COPY sources/ ./
COPY openapi/ /app/openapi/
RUN cargo build --release -p messenger

FROM alpine:3.20 AS runtime

RUN apk add --no-cache ca-certificates

WORKDIR /app

COPY webui/ /app/webui/
COPY --from=builder /app/sources/target/release/messenger /usr/local/bin/messenger

ENV MESSENGER_HTTP_ADDR=0.0.0.0:3030
ENV MESSENGER_WEBUI_DIR=/app/webui

EXPOSE 3030

CMD ["messenger"]
