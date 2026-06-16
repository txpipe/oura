#
# spec file for package oura
#
# Copyright (c) 2022 TxPipe
#
# All modifications and additions to the file contributed by third parties
# remain the property of their copyright owners, unless otherwise agreed
# upon. The license for this file, and modifications and additions to the
# file, is the same license as for the pristine package itself (unless the
# license for the pristine package is not an Open Source License, in which
# case the license is the MIT License). An "Open Source License" is a
# license that conforms to the Open Source Definition (Version 1.9)
# published by the Open Source Initiative.

# Please submit bugfixes or comments via https://github.com/txpipe/oura/issues
#

%global pkg_name oura
%global rustflags '-Clink-arg=-Wl,-z,relro,-z,now'
Name:           %{pkg_name}
Version:        1.0.2
Release:        0
Summary:	The tail of Cardano
License:        Apache-2.0
URL:            https://github.com/txpipe/oura
Source0:        %{name}-%{version}.tar.xz
Source1:        vendor.tar.xz
Source2:        cargo_config
BuildRequires:  cargo-packaging

%description
We have tools to "explore" the Cardano blockchain, which are useful when you know what you're looking for.
We argue that there's a different, complementary use-case which is to "observe" the blockchain and react
to particular event patterns.

%prep
%setup -qa1
mkdir .cargo
cp %{SOURCE2} .cargo/config

%build
%{cargo_build}

%install
# using cargo_install (only supports bindir)
%{cargo_install}

%check
%{cargo_test}

%files
%license LICENSE
%doc README.md
%{_bindir}/oura

%changelog

