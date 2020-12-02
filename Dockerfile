FROM registry.access.redhat.com/ubi8/ubi:latest as builder

RUN dnf -y update
RUN dnf -y install curl openssl-devel npm gcc gcc-c++ make cyrus-sasl-devel cmake libpq-devel

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y

ENV PATH "$PATH:/root/.cargo/bin"

WORKDIR /usr/src
COPY . /usr/src

RUN cargo build --release

FROM registry.access.redhat.com/ubi8-minimal

LABEL org.opencontainers.image.source="https://github.com/drogue-iot/watson-speech-to-text-converter"

COPY --from=builder /usr/src/target/release/watson-speech-to-text-converter /

ENTRYPOINT [ "/watson-speech-to-text-converter" ]
