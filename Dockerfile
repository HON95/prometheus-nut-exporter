ARG APP_VERSION=0.0.0-SNAPSHOT
ARG APP_GID=5000
ARG APP_UID=5000
ARG ALPINE_VERSION=3.16
ARG RUST_VERSION=1.62.0


## Builder stage
FROM alpine:$ALPINE_VERSION AS builder
WORKDIR /app

# Install Rust
RUN apk add --no-cache build-base curl
ARG RUST_VERSION
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | ash -s -- --default-toolchain=$RUST_VERSION -y
ENV PATH="$PATH:/root/.cargo/bin"

# Fetch deps using dummy app
COPY Cargo.toml ./
COPY Cargo.lock ./
RUN mkdir src \
&& echo "fn main() {}" > src/main.rs \
&& cargo fetch \
&& rm -rf src/

# Build app
COPY src/ src/
ARG APP_VERSION
RUN sed -i "s/^.*\bAPP_VERSION\b.*$/pub const APP_VERSION: \&str = \"$APP_VERSION\";/g" src/meta.rs
RUN cargo rustc --release -- -D warnings


## Runtime stage
FROM alpine:$ALPINE_VERSION AS runtime
WORKDIR /app

# Add non-root user
ARG APP_GID
ARG APP_UID
RUN addgroup -S -g $APP_GID app && adduser -S -G app -u $APP_UID app

# Add executable
COPY --from=builder /app/target/release/prometheus-nut-exporter ./
RUN chown app:app prometheus-nut-exporter

USER app
ENTRYPOINT ["./prometheus-nut-exporter"]
