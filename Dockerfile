# Note: "--platform=$BUILDPLATFORM" is required to avoid a QEMU build bug.

ARG APP_VERSION=0.0.0-SNAPSHOT
ARG APP_GID=5000
ARG APP_UID=5000
ARG APP_ENV=prod
ARG ALPINE_VERSION=3.16
ARG RUST_VERSION=1.62.0


## Build stage
FROM --platform=$BUILDPLATFORM alpine:$ALPINE_VERSION AS build
WORKDIR /app

# Install Rust
RUN apk add --no-cache build-base rustup
ARG RUST_VERSION
RUN rustup-init --default-toolchain=$RUST_VERSION -y
ENV PATH="$PATH:/root/.cargo/bin"

# Fetch deps using dummy app
COPY Cargo.toml ./
COPY Cargo.lock ./
RUN mkdir src \
&& echo "fn main() {}" > src/main.rs \
&& cargo fetch \
&& rm -rf src/

# Build real app
COPY src/ src/
# Set version
ARG APP_VERSION
RUN sed -i "s/^.*\bAPP_VERSION\b.*$/pub const APP_VERSION: \&str = \"$APP_VERSION\";/g" src/meta.rs
# Break on warnings if prod
ARG APP_ENV
RUN echo "Build env: $APP_ENV"; \
if [ "$APP_ENV" = "prod" ]; \
then cargo rustc --release -- -D warnings; \
else cargo rustc --release; \
fi


## Runtime stage
FROM --platform=$BUILDPLATFORM alpine:$ALPINE_VERSION AS runtime
WORKDIR /app

# Add non-root user
ARG APP_GID
ARG APP_UID
RUN addgroup -S -g $APP_GID app && adduser -S -G app -u $APP_UID app

# Add executable
COPY --from=build /app/target/release/prometheus-nut-exporter ./
RUN chown app:app prometheus-nut-exporter

USER app
ENTRYPOINT ["./prometheus-nut-exporter"]
