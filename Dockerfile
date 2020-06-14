FROM rust:1.43 as builder

ARG APP_ENV="prod"
ENV BINARY="prometheus-nut-exporter"

WORKDIR /build

COPY Cargo.toml ./
COPY Cargo.lock ./

# Fetch deps using dummy app
RUN mkdir src \
&& echo "fn main() {}" > src/main.rs \
&& cargo fetch \
&& rm -rf src/

# Build real app
COPY src/ src/
# Break on warnings if prod
RUN echo "Build env: $APP_ENV"; \
if [ "$APP_ENV" = "prod" ]; \
then cargo rustc --release -- -D warnings; \
else cargo rustc --release; \
fi

FROM debian:buster-slim as runner

ARG APP_UID=5000
ENV BINARY="prometheus-nut-exporter"

WORKDIR /app

RUN useradd --uid $APP_UID app

COPY --from=builder /build/target/release/$BINARY ./
RUN chown app:app $BINARY

USER app
EXPOSE 9999/tcp
ENTRYPOINT ./$BINARY
