FROM mcr.microsoft.com/devcontainers/rust:1-1-bullseye
RUN apt update
RUN apt install -y build-essential libtool libreadline-dev zlib1g-dev flex bison libseccomp-dev libssl-dev clang pkg-config libpq-dev cmake postgresql-client libprotobuf-dev libcurl4-openssl-dev openssl lsof libicu-dev
ARG PROTOC_VERSION=30.2
RUN curl -Lo protoc.zip https://github.com/protocolbuffers/protobuf/releases//download/v$PROTOC_VERSION/protoc-$PROTOC_VERSION-linux-x86_64.zip
RUN unzip -q protoc.zip bin/protoc -d /usr/local
RUN unzip -q protoc.zip include/google/protobuf/* -d /usr/local
RUN chmod a+x /usr/local/bin/protoc
RUN chmod a+x /usr/local/include/google/protobuf/
RUN rm -rf protoc.zip
ARG USERNAME=vscode
USER $USERNAME
