# Omnix Configuration

You can configure Omnix's behaviour on your repository by creating a top-level `om.yaml` file.  If there is no `om.yaml` file, Omnix will evaluate the flake's `.#om` output instead. Prefer creating a `om.yaml` in general, as it is faster to read than evaluating complex Nix flakes.

> [!NOTE]
> For am example `om.yaml`, see [that of omnix itself](https://github.com/juspay/omnix/blob/main/om.yaml)