FROM ekidd/rust-musl-builder:latest as builder

ADD . ./
RUN sudo chown -R rust:rust /home/rust
RUN cargo build --release --features all

FROM alpine:latest

WORKDIR /opt/dystopia

RUN mkdir /opt/dystopia/bin

COPY --from=builder \
  /home/rust/src/target/x86_64-unknown-linux-musl/release/dytp \
  /opt/dystopia/bin/

ENV PATH /opt/dystopia/bin:${PATH}

ENTRYPOINT ["dytp"]
