## 1Password Integration

The project can pull distcc host credentials directly from 1Password so secrets never land in source control.

### Requirements

- [1Password CLI (`op`)](https://developer.1password.com/docs/cli)
- Signed-in session (`op signin …`)
- Optional: Cursor 1Password extension (for inline secret references)

### Sync distcc host from 1Password

Use `scripts/op_sync_distcc_host.sh` to populate:

- `ansible/hosts`
- `~/.ssh/<alias>_id_ed25519` and SSH config
- `~/.distcc/hosts`
- `~/.zshrc` (`DISTCC_HOSTS` export)

```bash
export OP_DISTCC_HOST_SECRET="op://Engineering/Distcc M4/host"
export OP_DISTCC_USER_SECRET="op://Engineering/Distcc M4/username"
export OP_DISTCC_KEY_SECRET="op://Engineering/Distcc M4/private key"
# optional
export OP_DISTCC_CORES_SECRET="op://Engineering/Distcc M4/cores"
export DISTCC_REMOTE_ALIAS="distcc-m4"

./scripts/op_sync_distcc_host.sh
```

Then run the provisioning playbook:

```bash
ansible-playbook -i ansible/hosts ansible/playbooks/setup_distcc_macos.yml
```

### Cursor references

You can reference the same secrets inside Cursor prompts using the extension, e.g.:

```
op://Engineering/Distcc M4/host
op://Engineering/Distcc M4/username
op://Engineering/Distcc M4/private key
```

### Notes

- `OP_DISTCC_*` variables accept any 1Password item paths.
- The script rewrites `ansible/hosts` for the `distcc_macos_workers` group each run.
- Update `DISTCC_REMOTE_ALIAS` or `DISTCC_REMOTE_CORES` to match new hosts.
