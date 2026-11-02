# Docker

LACT is available as a Docker image on [GHCR](https://github.com/ilya-zlobintsev/LACT/pkgs/container/lact).

This image only contains the daemon and CLI, it does not have a GUI. 
The intended use case is for servers and headless systems, for example as a [metrics exporter](./EXPORTER.md).

# Usage

Basic usage:
```
docker run -d --name lact --privileged -v ./config:/etc/lact -v /dev/dri:/dev/dri ghcr.io/ilya-zlobintsev/lact:master
```

To call the CLI, use `docker exec`:
```
docker exec lact lact cli info
```

You can set GPU configuration options by editing `config.yaml` in the config dir mounted into the container.

# Nvidia

The image contains Nvidia support, but additional setup is needed to expose Nvidia GPUs through Docker.
Please check Nvidia's container documentation for how to set it up.

TODO: add nvidia command example
