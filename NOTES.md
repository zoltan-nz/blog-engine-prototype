# Implementation Notes and Findings

## Backend Resource Comparison

Comparing production Docker images after implementing minimal `/healthz` endpoints.

### Measurement Commands

```bash
podman image ls
podman stats --no-stream
```

### Results

| Container         | Image Size | Memory Usage |
|-------------------|------------|--------------|
| backend-rust-prod | 122 MB     | 0.75 MB      |
| backend-node-prod | 255 MB     | 26.5 MB      |

### Analysis

- **Image size**: Rust is ~2x smaller (122 MB vs 255 MB)
- **Memory usage**: Rust uses ~35x less RAM (0.75 MB vs 26.5 MB)
- **Note**: Node image includes Alpine + Node.js runtime + V8 engine; Rust compiles to a single static binary on Debian slim

