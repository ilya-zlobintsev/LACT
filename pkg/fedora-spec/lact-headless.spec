Name:           lact-headless
Version:        0.7.3
Release:        2
Summary:        AMDGPU control utility
License:        MIT
URL:            https://github.com/ilya-zlobintsev/LACT
Source0:        https://github.com/ilya-zlobintsev/LACT/archive/refs/tags/v0.7.3.tar.gz

BuildRoot:      %{_tmppath}/%{name}-%{version}-%{release}-root-%(%{__id_u} -n)
BuildRequires:  rust cargo gcc libdrm-devel dbus curl make clang git vulkan-tools
Requires:       libdrm hwdata vulkan-tools

%description
AMDGPU control utility

%prep
%setup -q -n LACT-%{version}

%build
VERGEN_GIT_SHA=a86139a make build-release-headless %{?_smp_mflags}

%install
rm -rf %{buildroot}
make install PREFIX=/usr DESTDIR=%{buildroot}

%files
%defattr(-,root,root,-)
%license LICENSE
%doc README.md
/usr/bin/lact
/usr/lib/systemd/system/lactd.service
/usr/share/applications/io.github.ilya_zlobintsev.LACT.desktop
/usr/share/icons/hicolor/scalable/apps/io.github.ilya_zlobintsev.LACT.svg
/usr/share/pixmaps/io.github.ilya_zlobintsev.LACT.png
/usr/share/metainfo/io.github.ilya_zlobintsev.LACT.metainfo.xml

%changelog
* Sat Apr 05 2025 - ilya-zlobintsev - v0.7.3 - v0.7.3
- Autogenerated from CI, please see  for detailed changelog.
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
