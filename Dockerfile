FROM ubuntu:20.04 AS build-doom

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update -y && \
    apt-get install -y \
    build-essential \
    libsdl-mixer1.2-dev \
    libsdl-net1.2-dev \
    git \
    gcc \
    wget

# Setup DOOM
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

# Setup a VNC password
RUN mkdir ~/.vnc && \
    x11vnc -storepasswd ${VNC_PASSWORD} ~/.vnc/passwd

WORKDIR /root

RUN bash -c 'echo "/usr/local/games/psdoom -warp E1M1 -iwad /doom1.wad" >> /root/.bashrc'
