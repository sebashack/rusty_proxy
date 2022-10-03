# Rusty Reverse Proxy Server

## Install dependencies

Execute:

```
./ first-time-install.sh
```

## Build binary

Execute:

```
make build
```

The binary's name is `rusty_proxy` and will be located at `<project-root>/dist`.


## Execute binary

```
./dist/rusty_proxy /path/to/config.yaml
```

There is an example config file at the root of this project named `example_config.yaml`.
