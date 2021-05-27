# this container builds the kilt-parachain binary from source files and the runtime library
# pinned the version to avoid build cache invalidation
FROM paritytech/ci-linux:d34c7950-20210408 as builder

WORKDIR /build

ARG FEATURES=default

COPY ./.git/ /build/.git/
COPY ./nodes /build/nodes
COPY ./pallets /build/pallets
COPY ./primitives /build/primitives
COPY ./runtimes /build/runtimes
COPY ./Cargo.lock /build/Cargo.lock
COPY ./Cargo.toml /build/Cargo.toml

RUN cargo build --release --features $FEATURES

# ===== SECOND STAGE ======

FROM debian:buster-slim
LABEL description="This is the 2nd stage: a very small image where we copy the kilt-parachain binary."

ARG NODE_TYPE=kilt-parachain

COPY ./LICENSE /build/LICENSE
COPY ./README.md /build/README.md
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
