# Updating sealed secret

While connected to the cluster, run `nix run nixpkgs#kubeseal -- -f _secret.yaml -w secret.yaml` to update the sealed secret. Note that the `namespace` field must match the namespace that the secret is deployed to.
