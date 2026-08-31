# Stage 1: Build the React + PatternFly web editor
FROM node:24.15-slim AS webapp-builder
WORKDIR /build
COPY webapp/package.json webapp/package-lock.json ./
RUN npm ci
COPY webapp/ ./
RUN npm run build

# Stage 2: Build the Rust binary with a glibc version compatible with UBI 9.
FROM rust:1.93-bullseye AS rust-builder
WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
COPY templates/ templates/
RUN cargo build --release

# Stage 3: Runtime image
FROM registry.access.redhat.com/ubi9/ubi-minimal

RUN microdnf install -y graphviz && \
    microdnf clean all

COPY --from=rust-builder /build/target/release/ai-arch-story /usr/local/bin/ai-arch-story
COPY --from=webapp-builder /build/dist /usr/share/ai-arch-story/webapp
COPY templates/ /usr/share/ai-arch-story/templates/

WORKDIR /workspace
EXPOSE 8080
ENTRYPOINT ["ai-arch-story"]
