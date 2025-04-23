FROM quay.io/rockylinux/rockylinux:8
RUN dnf install curl --allowerasing -y  # Fix for curl-minimal conflict
RUN dnf install -y dnf-plugins-core rocky-repos
RUN dnf config-manager --enable devel
