#!/bin/sh

# Clean up previous files.
rm -fr .rootfs
rm -f initrd

docker build -t ml-code-runner .
docker create --name cnt-ml-code-runner ml-code-runner /bin/sh
docker cp cnt-ml-code-runner:/ .rootfs
docker rm cnt-ml-code-runner

mkfs.erofs --all-root -d2 -E noinline_data initrd ./.rootfs
