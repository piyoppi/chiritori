# Chiritori Changelog

## 1.3.0

### Features

Add `--removal-marker-target-config` option. See [README](https://github.com/piyoppi/chiritori/blob/v1.3.0/README.md#removal-marker).

## 1.2.1

### Bug fixes

- Fixes an error failing to parse if there is a newline after an attribute (https://github.com/piyoppi/chiritori/pull/26)

## 1.2.0

### Features

- Add `--list-all` Option: The feature of displaying a list of source code to be removed or pending.

## 1.1.0

### Features

- Add `--list` Option: The feature of displaying a list of source code to be removed.

## 1.0.0

### Features

See README for details of the additional features.

- Add `marker` removal tag
- Add `unwrap-block` removal strategy
- Add `skip` attribute (to skip removal process)

### Breaking Changed

- Default delimiters is changed from `<!--`, `-->` to `<!-- <`, `> -->`

## 0.3.0

The formatting process performed after the deletion of time-limited content has been improved. (https://github.com/piyoppi/chiritori/pull/5, https://github.com/piyoppi/chiritori/pull/6)

If there is a blank line before and after the deletion position and a new line immediately after the deletion position, the new line immediately after the deletion position is deleted.

(original)

```text
foo
<!-- time-limited to="1999-12-31 23:59:59" -->
  content
<!-- /time-limited -->
bar
```

(converted)

```text
foo
bar
```

If there is a blank line before or after the delete position, the blank line is kept.

```text
foo

<!-- time-limited to="1999-12-31 23:59:59" -->
  content
<!-- /time-limited -->

bar
```

(converted)

```text
foo

bar
```

## 0.2.0

Stdin is only received from pipes (https://github.com/piyoppi/chiritori/pull/3)

## 0.1.0

Initial Release
