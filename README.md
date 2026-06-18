# quadctl

A small CLI for managing quadlets on remote nodes.

See tests/data for example quadlet definitions and inventory.toml examples.

The strategy is simple. Generate a hash file of each quadlet and their dependencies, compare it to the remote file,
push up any changed files, and restart any containers that need it.

The quadlets define their dependencies in an inventory.toml file and the sha256 hashes are computer for each file
if a file has dependencies then the hash of the file and all of its dependencies are concatenated together and
a new hash is generated for the parent file. In theory dependencies can be as deeply nested as desired.

Know limitation. Circular dependencies are not supported or prevented.
