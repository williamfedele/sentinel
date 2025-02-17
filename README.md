# Sentinel
Automatic command execution on file change.

## Running
By default, Sentinel watches the directory in which `sentinel` is ran. You can specify the dir with `--dir`.

```shell
sentinel --dir WATCH_DIR
```

### Configuration
Sentinel uses a yaml format for specifying which commands should be ran for file extensions. Each of the entries in `commands` is a file extension specifying which commands should be executed should a file with that extension be modified. The `{file}` placeholder is replaced by modified file names during runtime.

Example:
```yaml
commands:
  py:
    - "ruff check {file}"
    - "ruff format {file} -v"
  ts:
    - "prettier --write {file}"
```

You can also create a global config file in your platforms config directory:

- Linux: `$XDG_CONFIG_HOME/sentinel/global.yaml`
- Windows: `%APPDATA%/sentinel/global.yaml`
- MacOS: `$HOME/Library/Application Support/sentinel/global.yaml`

You can also specify a project level config `.sentinel.yaml` in the directory which will be watched by Sentinel. Project configs have priority over the global config.


## License

[MIT](https://github.com/williamfedele/sentinel/blob/main/LICENSE)
