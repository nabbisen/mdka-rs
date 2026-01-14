# mdka Development

## Where are files around multiple platforms support

`npm/` directory under here (`napi/`) will be automatically generated in GitHub Actions CI workflow with `napi create-npm-dir`.

## Maintenance

### Packages update

```console
$ npm update # update package-lock.json
```

### Just version modify

```console
$ npm version 0.0.0 # next version
```

## Supported platforms

### napi

Default with darwin x64 replaced with arm64 (ref: https://napi.rs/docs/cli/napi-config )
