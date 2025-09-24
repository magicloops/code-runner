# Code Runner on Unikraft Cloud

To deploy Code Runner on Unikraft Cloud, follow the steps below.

1. Make sure you have `erofs` installed.
   On a Debian-based distribution, use:

   ```console
   sudo apt update
   sudo apt install -y erofs-utils
   ```

1. Have [Docker](/home/razvan/orgs/unikraft-io/pocs/tinyfish/chromium-cdp) and [KraftKit](https://unikraft.org/docs/cli/install) installed.

1. Create root filesystem as an initrd file and use `erofs` (Extended Read-only Filesystem):

   ```console
   ./create-initrd.sh
   ```

1. Deploy Code Runner on Unikraft Cloud:

   ```console
   kraft cloud deploy --kraftfile Kraftfile.erofs --port 443:4000 --memory 1Gi --name code-runner --subdomain code-runner .
   ```
