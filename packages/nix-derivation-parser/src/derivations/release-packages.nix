/*
  This file is copied from github.com/nixos/nixpkgs.
  Originally from rev:
    97ebfb68e5e32bceea415f6a5f670f026634bd4d
  Specifically from this file:
    github.com/nixos/nixpkgs/pkgs/top-level/release-small.nix

  Can be used with `nix-instantiate` with an argument passed to `path` pointing
  to a checkout of nixpkgs.
*/

{
  path ? ./.,
  caDrvs ? false,
  lib ? import (path + /lib),
  nixpkgs ? {
    outPath = lib.cleanSource path;
    revCount = 1234;
    shortRev = "abcdef";
  },
  supportedSystems ? [
    "x86_64-linux"
    "x86_64-darwin"
    "aarch64-linux"
    "aarch64-darwin"
  ],
  # Attributes passed to nixpkgs. Don't build packages marked as unfree.
  nixpkgsArgs ? {
    config = {
      allowUnfree = false;
      inHydra = true;
      contentAddressedByDefault = caDrvs;
    };
  },
}:

let
  release-lib = import (path + /pkgs/top-level/release-lib.nix) {
    inherit supportedSystems nixpkgsArgs;
  };

  inherit (release-lib)
    all
    linux
    darwin
    mapTestOn
    unix
    ;

  inherit (lib)
    attrNames
    foldl'
    isAttrs
    mapAttrs
    ;

  getDrvPaths = mapAttrs (_: pkgsSet: mapAttrs (_: pkgByPlatform: pkgByPlatform.drvPath) pkgsSet);
  flattenAttrs =
    parent: attrs:
    let
      op =
        acc: n:
        let
          v = attrs.${n};
          x =
            if (isAttrs v) then
              if parent == "" then flattenAttrs n v else flattenAttrs "${parent}-${n}" v
            else
              { "${parent}-${n}" = v; };
        in
        acc // x;
    in
    foldl' op { } (attrNames attrs);
in

{
  pkgSet = flattenAttrs "" (
    getDrvPaths (mapTestOn {
      aspell = all;
      at = linux;
      autoconf = all;
      automake = all;
      avahi = unix; # Cygwin builds fail
      bash = all;
      bashInteractive = all;
      bc = all;
      binutils = linux;
      bind = linux;
      bsdiff = all;
      bzip2 = all;
      cmake = all;
      coreutils = all;
      cpio = all;
      cron = linux;
      cups = linux;
      dbus = linux;
      diffutils = all;
      e2fsprogs = linux;
      emacs = linux;
      file = all;
      findutils = all;
      flex = all;
      gcc = all;
      glibc = linux;
      glibcLocales = linux;
      gnugrep = all;
      gnum4 = all;
      gnumake = all;
      gnupatch = all;
      gnupg = linux;
      gnuplot = unix; # Cygwin builds fail
      gnused = all;
      gnutar = all;
      gnutls = linux;
      grub2 = linux;
      guile = linux; # tests fail on Cygwin
      gzip = all;
      hddtemp = linux;
      hdparm = linux;
      hello = all;
      host = linux;
      iana-etc = linux;
      icewm = linux;
      idutils = all;
      inetutils = linux;
      iputils = linux;
      qemu = linux;
      qemu_kvm = linux;
      lapack-reference = linux;
      less = all;
      lftp = all;
      libtool = all;
      libtool_2 = all;
      libxml2 = all;
      libxslt = all;
      lout = linux;
      lsof = linux;
      ltrace = linux;
      lvm2 = linux;
      lynx = linux;
      xz = linux;
      man = linux;
      man-pages = linux;
      mc = all;
      mdadm = linux;
      mesa = linux;
      mingetty = linux;
      mktemp = all;
      monotone = linux;
      mutt = linux;
      netcat = linux; # netcat broken on darwin
      nfs-utils = linux;
      nix = all;
      nss_ldap = linux;
      nssmdns = linux;
      ntfs3g = linux;
      ntp = linux;
      openssh = linux;
      openssl = all;
      pan = linux;
      pciutils = linux;
      perl = all;
      pkg-config = all;
      pmccabe = linux;
      procps = linux;
      python3 = unix; # Cygwin builds fail
      readline = all;
      rlwrap = all;
      rpcbind = linux;
      rsync = linux;
      screen = linux ++ darwin;
      scrot = linux;
      sdparm = linux;
      smartmontools = all;
      sqlite = unix; # Cygwin builds fail
      msmtp = linux;
      stdenv = all;
      strace = linux;
      su = linux;
      sudo = linux;
      sysklogd = linux;
      tcl = linux;
      tcpdump = linux;
      texinfo = all;
      time = linux;
      tinycc = linux;
      udev = linux;
      unzip = all;
      usbutils = linux;
      util-linux = linux;
      util-linuxMinimal = linux;
      w3m = all;
      webkitgtk_4_0 = linux;
      wget = all;
      which = all;
      wirelesstools = linux;
      wpa_supplicant = linux;
      xfsprogs = linux;
      xkeyboard_config = linux;
      zip = all;
    })
  );
}
