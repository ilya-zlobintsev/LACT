FROM quay.io/rockylinux/rockylinux:9
RUN dnf install curl --allowerasing -y # Fix for curl-minimal conflict
RUN dnf install -y dnf-plugins-core rocky-repos epel-release
RUN dnf config-manager --enable devel
