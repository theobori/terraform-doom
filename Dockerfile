FROM rust:latest AS build-tf-doom

ENV DIRNAME tfdoom

WORKDIR ${DIRNAME}

COPY . .

RUN cargo build --release
RUN mv target/release/tf-doom /

RUN rm -rf /${DIRNAME}

FROM ubuntu:20.04 AS build-doom

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update -y && \
    apt-get install -y \
    build-essential \
    libsdl-mixer1.2-dev \
    libsdl-net1.2-dev \
    git \
    gcc \
    unzip \
    wget

# Installing Terraform
RUN wget --quiet -O terraform.zip https://releases.hashicorp.com/terraform/1.4.6/terraform_1.4.6_linux_amd64.zip \
    && unzip terraform.zip \
    && mv terraform /usr/bin \
    && rm terraform.zip

# Installing the DOOM IWAD + dockerdoom made by GideonRed
RUN git clone https://github.com/GideonRed/dockerdoom.git
RUN wget http://distro.ibiblio.org/pub/linux/distributions/slitaz/sources/packages/d/doom1.wad

WORKDIR /dockerdoom/trunk
RUN ./configure && \
    make && \
    make install

FROM ubuntu:20.04 AS run-doom

ARG VNC_PASSWORD=1234

RUN apt-get update -y && \
    apt-get install -y \
    x11vnc \
    xvfb \
    libsdl-mixer1.2 \
    libsdl-net1.2 \
    netcat

COPY --from=build-doom /doom1.wad /
COPY --from=build-doom /usr/local/games/psdoom /usr/local/games
COPY --from=build-doom /usr/bin/terraform /usr/bin
COPY --from=build-tf-doom /tf-doom /usr/bin

# Setup a VNC password
RUN mkdir /tf && \
    mkdir ~/.vnc && \
    x11vnc -storepasswd ${VNC_PASSWORD} ~/.vnc/passwd

ENTRYPOINT [ "/usr/bin/tf-doom" ]
