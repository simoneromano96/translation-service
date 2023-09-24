FROM debian as install-libtorch

RUN apt-get update && apt-get install -y curl unzip
RUN mkdir -p /vendor
WORKDIR /vendor

RUN curl https://download.pytorch.org/libtorch/cpu/libtorch-cxx11-abi-shared-with-deps-2.0.1%2Bcpu.zip -o /vendor/libtorch.zip
RUN unzip /vendor/libtorch.zip
RUN rm -rf /vendor/libtorch.zip

FROM debian as download-models

RUN apt-get update && apt-get install -y git git-lfs
RUN mkdir -p /models
WORKDIR /models

RUN git lfs install
RUN GIT_LFS_SKIP_SMUDGE=1 git clone https://huggingface.co/Helsinki-NLP/opus-mt-en-ROMANCE
RUN cd /models/opus-mt-en-ROMANCE && git lfs pull
RUN GIT_LFS_SKIP_SMUDGE=1 git clone https://huggingface.co/Helsinki-NLP/opus-mt-ROMANCE-en
RUN cd /models/opus-mt-ROMANCE-en && git lfs pull

FROM rust:slim as builder

RUN apt-get update && apt-get install -y pkg-config build-essential

RUN mkdir -p /vendor/libtorch
COPY --from=install-libtorch /vendor/libtorch /vendor/libtorch

ENV LIBTORCH=/vendor/libtorch
ENV LD_LIBRARY_PATH=/vendor/libtorch/lib:$LD_LIBRARY_PATH

WORKDIR /app

COPY . .

RUN cargo build --release

FROM debian

RUN apt-get update && apt-get install -y pkg-config libgomp1

WORKDIR /app

ENV APP_PATH=/app/models
ENV LIBTORCH=/vendor/libtorch
ENV LD_LIBRARY_PATH=/vendor/libtorch/lib:$LD_LIBRARY_PATH

RUN mkdir -p /vendor/libtorch

COPY --from=install-libtorch /vendor/libtorch /vendor/libtorch
COPY --from=download-models /models /app/models
COPY --from=builder /app/target/release/translation /app/translation

EXPOSE 8080

CMD ["/app/translation"]
