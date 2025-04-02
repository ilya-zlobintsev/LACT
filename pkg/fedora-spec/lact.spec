Name:           lact
Version:        0.7.2
Release:        1
Summary:        AMDGPU control utility
License:        MIT
URL:            https://github.com/ilya-zlobintsev/LACT
Source0:        https://github.com/ilya-zlobintsev/LACT/archive/refs/tags/v0.7.2.tar.gz

BuildRoot:      %{_tmppath}/%{name}-%{version}-%{release}-root-%(%{__id_u} -n)
BuildRequires:  rust cargo gtk4-devel gcc libdrm-devel dbus curl make clang git vulkan-tools
Requires:       gtk4 libdrm hwdata vulkan-tools

%description
AMDGPU control utility

%prep
%setup -q -n LACT-%{version}

%build
make build-release %{?_smp_mflags}

%install
rm -rf %{buildroot}
VERGEN_GIT_SHA=942526e make install PREFIX=/usr DESTDIR=%{buildroot}

%files
%defattr(-,root,root,-)
%license LICENSE
%doc README.md
/usr/bin/lact
/usr/lib/systemd/system/lactd.service
/usr/share/applications/io.github.ilya-zlobintsev.LACT.desktop
/usr/share/icons/hicolor/scalable/apps/io.github.ilya-zlobintsev.LACT.svg
/usr/share/pixmaps/io.github.ilya-zlobintsev.LACT.png

%changelog
* Sun Mar 16 2025 - ilya-zlobintsev - v0.7.2 - v0.7.2
- Autogenerated from CI, please see  for detailed changelog.
* Thu Feb 27 2025 - ilya-zlobintsev - v0.7.1 - v0.7.1
- Autogenerated from CI, please see  for detailed changelog.
* Wed Jan 15 2025 - ilya-zlobintsev -  - 
- Autogenerated from CI, please see  for detailed changelog.
* Thu Nov 14 2024 - ilya-zlobintsev -  - 
- Autogenerated from CI, please see  for detailed changelog.
* Thu Nov 14 2024 - ilya-zlobintsev -  - 
- Autogenerated from CI, please see  for detailed changelog.
