# Stellaux_Website

Customer-facing Stellaux website and backend domains.

## Unified Documentation Gateway

- Local portal: [../assets/docs-gateway/index.html](../assets/docs-gateway/index.html)
- Manifest source: [../assets/docs-gateway/docs-manifest.json](../assets/docs-gateway/docs-manifest.json)

## Notes

The docs gateway is generated from all Stellaux_*/docs folders in this workspace.
From workspace root, refresh it with:

```sh
node scripts/generate-docs-manifest.mjs
```
