FROM rustembedded/cross:x86_64-unknown-linux-gnu-0.2.1

COPY . .

RUN apt-get update && \
    apt-get install --assume-yes libx11-xcb-dev libgl1-mesa-dev xorg-dev openssl libssl-dev pkg-config
