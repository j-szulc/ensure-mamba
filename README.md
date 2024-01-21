# ensure-mamba

A simple wrapper around micromamba to ensure that it is installed and that some environment exists.
If it is not installed, it will be downloaded into `~/micromamba`.
Apart from that, once installed, it will forward all arguments to micromamba.

### Example

```bash
ensure-mamba install -n conda-forge python
```
