# QuadCTL

QuadCTL is a CLI tool for managing quadlets and their deployment.

The first step to making this work was to convert my existing homelab over to using rootless quadlets instead of rootful 
quadlets. This means that this cli will be able to use an existing ssh-copy-id key to manage the remote host without ever 
needing sudo access or having to enter a password.

I've laid out the inventory.toml file and manifests architecture.

I need to implement hashing on the file including it's dependencies. The dependencies are always specified in the
inventory.toml file instead of being derived from anything.

Secrets aren't part of this project at the moment. I'll leveraging systemd overrides to manage secrets directly on the
server. Maybe later they can be managed via sops. I'm not sure yet.

The process should be pretty simple. Compute the hashes locally. Download the remote hashes via ssh and compare them
to the local hashes. If they don't match, scp the file to the remote host and restart the service. Finally, upload the
new computed hashes to the remote host.

The file in the remote should be in $XDG_STATE_HOME/quadctl.toml or .local/state/quadctl.toml.

The state only cares about the .container files since those are the ones that need to be restart if they or their
dependenies change.

## Future

I'd like for quadctl to support updating images or at least checking for updates and printing out a report.
