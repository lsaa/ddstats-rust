# Funny grpc

PROTOC_ZIP=protoc-3.7.1-linux-x86_64.zip
curl -OL https://github.com/protocolbuffers/protobuf/releases/download/v3.7.1/$PROTOC_ZIP
sudo unzip -o $PROTOC_ZIP -d /usr/local bin/protoc
sudo unzip -o $PROTOC_ZIP -d /usr/local 'include/*'
rm -f $PROTOC_ZIP

set -x

wget --no-check-certificate https://xcb.freedesktop.org/dist/xcb-proto-1.11.tar.bz2
wget --no-check-certificate https://xcb.freedesktop.org/dist/libxcb-1.11.1.tar.bz2

tar -xjf xcb-proto-1.11.tar.bz2
cd xcb-proto-1.11
./configure
make
sudo make install
cd ..

tar -xjf libxcb-1.11.1.tar.bz2
cd libxcb-1.11.1
./configure --enable-xkb --enable-xinput --enable-selinux
make
sudo make install
cd ..

sudo apt-get install libx11-xcb-dev libgl1-mesa-dev
sudo apt install xorg-dev
sudo apt install openssl libssl-dev pkg-config
