FROM debian as install-libtorch

RUN apt-get update && apt-get install -y curl unzip
RUN mkdir -p /vendor
WORKDIR /vendor

RUN curl https://download.pytorch.org/libtorch/cpu/libtorch-cxx11-abi-shared-with-deps-2.0.1%2Bcpu.zip -o /vendor/libtorch.zip
RUN unzip /vendor/libtorch.zip
RUN rm -rf /vendor/libtorch.zip

FROM rust:slim as builder

RUN apt-get update && apt-get install -y pkg-config libssl-dev build-essential

RUN mkdir -p /vendor/libtorch
COPY --from=install-libtorch /vendor/libtorch /vendor/libtorch

ENV LIBTORCH=/vendor/libtorch
ENV LD_LIBRARY_PATH=/vendor/libtorch/lib:$LD_LIBRARY_PATH

WORKDIR /app

COPY . .

RUN cargo build --release

FROM debian

RUN mkdir -p /vendor/libtorch
COPY --from=install-libtorch /vendor/libtorch /vendor/libtorch

ENV LIBTORCH=/vendor/libtorch
ENV LD_LIBRARY_PATH=/vendor/libtorch/lib:$LD_LIBRARY_PATH

RUN apt-get update && apt-get install -y pkg-config libssl-dev

WORKDIR /app

EXPOSE 80

COPY --from=builder /app/target/release/translation ./translation

CMD ["./translation"]
