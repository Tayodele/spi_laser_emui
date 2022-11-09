FROM arm32v7/rust:1-slim-buster

ARG USER="freeform"
ENV SOUND_FLOW_VERSION="0.4"

RUN apt-get update; \
    apt-get -y install git \
    build-essential \
    sudo

RUN useradd -m ${USER} && \
    usermod -G sudo ${USER} && \
    mkdir -p /home/${USER} && \
    echo "${USER}: " | chpasswd && \
    chown -R ${USER}:${USER} /home/${USER}

USER ${USER}