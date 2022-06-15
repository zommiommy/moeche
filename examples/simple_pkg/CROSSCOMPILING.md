# CROSS COMPILING

To cross-compile for all os and archs you need to create a Docker image
(Based on `DockerFileManyLinux2010`  which is in the root of the repository).
This Docker Immage should have the following folders:
 - `/build/{target-triple}/include` the headers to include for the python version we are using to compile. This is used for PYO3_CROSS_INCLUDE_DIR.
 - `/build/{target-triple}/lib` the **compiled** libpython DSO for this triple.

This docker should have the name `cross_compile_moeche`.
Then you can crosscompile everything with:
```bash
moeche wheel ./recipes/cross.yml publish
```

Everything is configurable inside the `cross.yml` file.