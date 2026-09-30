# Styx Containerized Agent Harness & Mission Control
FROM fedora:latest

WORKDIR /app

# Copy host-built binary, UI distribution, and markdown memory
COPY target/release/styx /usr/local/bin/styx
COPY ui/dist /app/ui/dist
COPY memory/ /app/memory/
COPY examples/ /app/examples/

ENV STYX_HOST=0.0.0.0 \
    STYX_PORT=3000 \
    STYX_MEMORY_DIR=/app/memory \
    STYX_DATA_DIR=/app/data \
    STYX_STATIC_DIR=/app/ui/dist \
    DOCKER_SOCKET=/var/run/docker.sock \
    STYX_MAX_CONCURRENT_TURNS=1

EXPOSE 3000
VOLUME ["/app/memory", "/app/data"]

CMD ["/usr/local/bin/styx"]
