# 0.3.0

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

# 0.2.0

Stdin is only received from pipes (https://github.com/piyoppi/chiritori/pull/3)

# 0.1.0

Initial Release
