# Syndae Containerized Agent Harness & Mission Control
FROM fedora:latest

WORKDIR /app

# Copy host-built binary, UI distribution, and markdown memory
COPY target/release/syndae /usr/local/bin/syndae
COPY ui/dist /app/ui/dist
COPY memory/ /app/memory/
COPY examples/ /app/examples/

ENV SYNDAE_HOST=0.0.0.0 \
    SYNDAE_PORT=3000 \
    SYNDAE_MEMORY_DIR=/app/memory \
    SYNDAE_DATA_DIR=/app/data \
    SYNDAE_STATIC_DIR=/app/ui/dist \
    DOCKER_SOCKET=/var/run/docker.sock \
    SYNDAE_MAX_CONCURRENT_TURNS=1

EXPOSE 3000
VOLUME ["/app/memory", "/app/data"]

CMD ["/usr/local/bin/syndae"]
