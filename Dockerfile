ARG APP_VERSION=0.0.0-SNAPSHOT
ARG APP_GID=5000
ARG APP_UID=5000
ARG APP_ENV=prod

## Build stage
FROM --platform=$BUILDPLATFORM rust:1.62-slim-bullseye AS build
WORKDIR /app

COPY Cargo.toml ./
COPY Cargo.lock ./

# Fetch deps using dummy app
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
FROM --platform=$BUILDPLATFORM debian:11-slim AS runtime
# Default log level
ENV RUST_LOG=info
WORKDIR /app

# Add non-root user
ARG APP_GID
ARG APP_UID
RUN addgroup --system --gid $APP_GID app && adduser --system --ingroup app --uid $APP_UID app

# Add executable
COPY --from=build /app/target/release/prometheus-nut-exporter ./
RUN chown app:app prometheus-nut-exporter

USER app
ENTRYPOINT ["./prometheus-nut-exporter"]
