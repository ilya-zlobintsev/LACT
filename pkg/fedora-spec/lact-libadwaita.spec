Name:           lact-libadwaita
Version:        0.6.0
Release:        1
Summary:        AMDGPU control utility
License:        MIT
URL:            https://github.com/ilya-zlobintsev/LACT
Source0:        https://github.com/ilya-zlobintsev/LACT/archive/refs/tags/v0.6.0.tar.gz

BuildRoot:      %{_tmppath}/%{name}-%{version}-%{release}-root-%(%{__id_u} -n)
BuildRequires:  rust cargo gtk4-devel gcc libdrm-devel blueprint-compiler libadwaita-devel dbus curl make clang git
Requires:       gtk4 libdrm libadwaita hwdata

%description
AMDGPU control utility

%prep
%setup -q -n LACT-%{version}

%build
make build-release-libadwaita %{?_smp_mflags}

%install
rm -rf %{buildroot}
make install PREFIX=/usr DESTDIR=%{buildroot}

%files
%defattr(-,root,root,-)
%license LICENSE
%doc README.md
/usr/bin/lact
/usr/lib/systemd/system/lactd.service
/usr/share/applications/io.github.lact-linux.desktop
/usr/share/icons/hicolor/scalable/apps/io.github.lact-linux.svg
/usr/share/pixmaps/io.github.lact-linux.png

%changelog
* Thu Nov 14 2024 - ilya-zlobintsev -  - 
- Autogenerated from CI, please see  for detailed changelog.
* Thu Nov 14 2024 - ilya-zlobintsev -  - 
- Autogenerated from CI, please see  for detailed changelog.