FROM rustembedded/cross:x86_64-unknown-linux-gnu-0.2.1

RUN apt-get update && apt-get intall software-properties-common
RUN add-apt-repository universe
RUN apt-get update && \
    apt-get install --assume-yes libx11-xcb-dev libgl1-mesa-dev xorg-dev openssl libssl-dev pkg-config python3 mold
