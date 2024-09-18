FROM ubuntu:22.04
RUN mkdir /root/.cargo
COPY ./ /root/oblivion-rust
COPY ./rsproxy.config /root/.cargo/config
COPY ./ubuntu.list /etc/apt/source.list
RUN apt-get update && apt-get install rustc cargo -y
WORKDIR /root/oblivion-rust
ENTRYPOINT ["cargo", "run", "--release"]
