# ropen

This is a tool which automates the process of copying back figures from an ssh session and opening
them locally without X forwarding. The idea is sort of like [rmate](https://github.com/textmate/rmate).

When you call `ropen <e.g. figure-1.png>` from within your SSH session, it'll copy the file back to
your workstation and run `xdg-open` on it, so that you can see the figure. For small images and stuff,
this is much faster than X11.

Unlike `rmate`, this doesn't copy back the file if you edit it. It's purely designed for visualization. When
`xdg-open` returns, it deletes the copy of the file.

Installation
------------
1. Run the ropen "server" on your desktop. I use systemd for this, but you could run it in the background or something however you like.

My `home.nix` has this snippet:

```
    systemd.user.services.ropen = {
    Unit = {
      After = [ "graphical-session-pre.target" ];
      PartOf = [ "graphical-session.target" ];
    };
    Install = { WantedBy = [ "graphical-session.target" ]; };
    Service = {
      ExecStart = "${pkgs.ropen}/bin/server";
      Restart = "on-failure";
      RestartSec = 3;
    };
  };
```

2. SSH to your remote srvers with remote forwarding of 40877: `ssh -R 40877:localhost:40877 user@example.org`.

This can be made perminant by adding a section to your your `~/.ssh/config`:
```
Host <host>
  RemoteForward [localhost]:40877 [localhost]:40877
```

3. Install the `ropen` binary on your remote workstation.
4. Profit.


Usage
-----

```
$ ropen file
```
or to use a different app other than xdg-open:

```
$ ropen file app-other-than-xdg-open
```


Security
--------
Obviously this is a remote shell. It's fundamentally insecure by design. The server is configured to only listen
on the local interface, which I  suppose helps security a little bit if you were silly enough to put your local
desktop on the public internet without a firewall, but stil. Other than that, there's no security or authentication.
