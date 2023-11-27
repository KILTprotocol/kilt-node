# this container builds the kilt-parachain binary from source files and the runtime library
# pinned the version to avoid build cache invalidation

FROM paritytech/ci-unified:bullseye-1.70.0 as builder

WORKDIR /build

ARG FEATURES=default
ARG BINARY=kilt-parachain

COPY . .

RUN cargo build --locked --release --features $FEATURES -p $BINARY

# ===== SECOND STAGE ======

FROM docker.io/library/ubuntu:20.04

ARG BINARY=kilt-parachain

LABEL description="This is the 2nd stage: a very small image where we copy the ${BINARY} binary."

COPY --from=builder /build/target/release/$BINARY /usr/local/bin/node-executable

RUN useradd -m -u 1000 -U -s /bin/sh -d /node node && \
	mkdir -p /node/.local/share/node && \
	chown -R node:node /node/.local && \
	ln -s /node/.local/share/node /data


USER node
EXPOSE 30333 9933 9944
VOLUME ["/data"]

COPY ./dev-specs /node/dev-specs

ENTRYPOINT ["/usr/local/bin/node-executable"]
CMD ["--help"]
