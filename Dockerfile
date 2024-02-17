# This file is sourced from https://github.com/paritytech/polkadot-sdk/blob/master/docker/dockerfiles/polkadot/polkadot_builder.Dockerfile
FROM docker.io/paritytech/ci-linux:production as builder
# Require the current commit hash to be provided as an argument
ARG SUBSTRATE_CLI_GIT_COMMIT_HASH
RUN test -n "$SUBSTRATE_CLI_GIT_COMMIT_HASH" || (echo "SUBSTRATE_CLI_GIT_COMMIT_HASH not set, provide via --build-arg SUBSTRATE_CLI_GIT_COMMIT_HASH=(git rev-parse --short=11 HEAD)" && false)
ENV SUBSTRATE_CLI_GIT_COMMIT_HASH=$SUBSTRATE_CLI_GIT_COMMIT_HASH
WORKDIR /pop
COPY . /pop
RUN cargo build --release

# Build image
FROM debian:bullseye-slim as collator
COPY --from=builder /pop/target/release/pop-node /usr/bin/pop-node
CMD ["/usr/bin/pop-node"]
