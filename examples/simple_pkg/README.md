# Simple pkg
This crate compiles a simple pkg that exports a function that sums two numbers.

In `./recipes` we have several standard build systems that can be run with: 
```bash
# linux
moeche wheel ./recipes/linux.yml develop
# windows
moeche wheel ./recipes/windows.yml develop
# macos
moeche wheel ./recipes/darwin.yml develop
# apple m1
moeche wheel ./recipes/m1.yml develop
```

You can also use `moeche wheel ./path/to/recipe.yml publish` to builld a compatible version and upload it to PyPi.

we also have an experimental recipe to crosscompile but it requioes significant
configuration. See `CROSSCOMPILING.md` for more info.

