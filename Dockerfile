# this container builds the kilt-parachain binary from source files and the runtime library
# pinned the version to avoid build cache invalidation

# Corresponds to paritytech/ci-linux:production at the time of this PR
# https://hub.docker.com/layers/ci-linux/paritytech/ci-linux/production/images/sha256-4e8c072ea12bc17d99cb531adb58dea5a4c7d4880a8a86525052d24d1454e89e?context=explore
FROM paritytech/ci-linux@sha256:4e8c072ea12bc17d99cb531adb58dea5a4c7d4880a8a86525052d24d1454e89e as builder

WORKDIR /build

ARG FEATURES=default

COPY . .

RUN cargo build --locked --release --features $FEATURES

# ===== SECOND STAGE ======

FROM docker.io/library/ubuntu:20.04
LABEL description="This is the 2nd stage: a very small image where we copy the kilt-parachain binary."

ARG NODE_TYPE=kilt-parachain

COPY --from=builder /build/target/release/$NODE_TYPE /usr/local/bin/node-executable

RUN useradd -m -u 1000 -U -s /bin/sh -d /node node && \
	mkdir -p /node/.local/share/node && \
	chown -R node:node /node/.local && \
	ln -s /node/.local/share/node /data && \
	rm -rf /usr/bin /usr/sbin

USER node
EXPOSE 30333 9933 9944
VOLUME ["/data"]

COPY ./dev-specs /node/dev-specs

ENTRYPOINT ["/usr/local/bin/node-executable"]
CMD ["--help"]
