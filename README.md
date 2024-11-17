# 🧹 Chiritori

<img src="./images/chiritori.png" alt="logo" width="400" height="323">

'Chiritori' is a tool for removing time-limited source code.
The application can pre-tag source code (e.g. using comments) and remove them when required.

- [Install](#install)
- [Getting Started](#getting-started)
- [Command Arguments](#command-arguments)
- [How to pre-tag source code](#how-to-pre-tag-source-code)
  - [Delimiter Settings](#delimiter-settings)
- [Tags](#tags)
  - [`time-limited`](#time-limited)
  - [`removal-marker`](#removal-marker)
- [Removal Strategy](#removal-strategy)
  - [Range](#range-default)
  - [Unwrap Block](#unwrap-block)
  - [Skip](#skip)
- [Supported Character Code](#supported-character-code)

We also provide [actions-chiritori](https://github.com/piyoppi/actions-chiritori), available on GitHub Actions.

## Install

### Homebrew

```
$ brew tap piyoppi/tap
$ brew install chiritori
```

### Manually

Binaries are available on [GitHub Releases](https://github.com/piyoppi/chiritori/releases).

## Getting Started

You can try Chiritori with the source code in the [/sample directory](./samples/) of this repository.

## Command Arguments

Target source code can be provided to Chiritori via command line arguments or standard input.

```
$ chiritori --filename=./source-code.js

$ cat ./source-code.js | chiritori
```

If the `--output` option is not specified, the processed source code is output to standard output.

```
# The processed source code is written to filesystem.
$ chiritori --filename=./source-code.js --output=./processed-source-code.js

# The processed source code is output to standard output.
$ chiritori --filename=./source-code.js
```

More information on Command Line Interface arguments can be found in the `chiritori --help` command.

```
$ chiritori --help

A tool for removing time-limited source code

Usage: chiritori [OPTIONS]

Options:
  -f, --filename <FILENAME>
          The filename to read
  -o, --output <OUTPUT>
          The filename to write
      --delimiter-start <DELIMITER_START>
          The delimiter start [default: &lt;!-- &lt;]
      --delimiter-end <DELIMITER_END>
          The delimiter end [default: &gt; --&gt;]
      --time-limited-tag-name <TIME_LIMITED_TAG_NAME>
          The tag name for time-limited content [default: time-limited]
      --time-limited-time-offset <TIME_LIMITED_TIME_OFFSET>
          The time offset for time-limited content [default: +00:00]
      --time-limited-current <TIME_LIMITED_CURRENT>
          The current time for time-limited content [default: ]
      --removal-marker-tag-name <REMOVAL_MARKER_TAG_NAME>
          The tag name for removal-marker [default: removal-marker]
      --removal-marker-target-name <REMOVAL_MARKER_TARGET_NAME>
          Name of removal-marker to be removed [default: vec![]]
  -l, --list
          List source code to be removed
      --list-all
          List source code to be removed or pending
  -h, --help
          Print help
  -V, --version
          Print version
```

## How to pre-tag source code

Chiritori detects removable source code by "tag" in the source code.
Tag is enclosed by start and end delimiters and consist of a tag name and attributes.

The start and end delimiters can be set to any string.
```
           <!-- < tag-name attribute1="value"  > -->
           ^^^^^^ ^^^^^^^^ ^^^^^^^^^^^^^^^^^^  ^^^^^
  start-delimiter tag name      attribute      end-delimiter
```

The source code to be removed is enclosed in start and end tags.
The closing tag is the tag name with a slash prefixed.

```
<!-- <tag-name attribute1="value"> -->
Removal source code
<!-- </tag-name> -->
```

### Delimiter Settings

The delimiter for representing an element can be changed via command line arguments.
This allows any programming language to be used.

```
chiritori \
  --delimiter-start="// --" \
  --delimiter-end="-- //" \
```

## Tags

### `time-limited`

Source code enclosed in `time-limited` tags is removed after the specified time.

#### Attributes

| Name | Detail            | Example                |
| ---  | ---               | ---                    |
| to   | Expiration Time   | 2024-01-01 00:00:00    |

#### Example

```html
<html>
  <body>
    <h1>Hello World</h1>
    <!-- <time-limited to="2024/02/15 12:00:00"> -->
    <h2>Campaign until 2024/02/15 12:00</h2>
    <!-- </time-limited> -->
    <!-- <time-limited to="9999/12/31 23:59:59"> -->
    <h2>Campaign until 9999/12/31</h2>
    <!-- </time-limited> -->
  </body>
</html>
```

### `removal-marker`

If the command line argument `--removal-marker-target-name` is specified, tags whose name attribute matches the value of the argument are targeted for deletion.

```
chiritori --filename=./samples/sample-code.js --removal-marker-target-name="feature1"
```

#### Attributes

| Name   | Detail                   | Example  |
| ---    | ---                      | ---      |
| name   | Marker Name (any string) | Feature1 |

#### Example

```html
<html>
  <body>
    <h1>Hello World</h1>
    <!-- <removal-marker name="Feature1"> -->
    <p>Feature 1 will be released soon.</p>
    <!-- </removal-marker> -->
    <!-- <removal-marker name="Feature2"> -->
    <p>Feature 2 will be released soon.</p>
    <!-- </removal-marker> -->
  </body>
</html>
```

## Removal Strategy

Chiritori has several removal strategies.
You can choose a strategy by argument (except for the default strategy).

### Range (Default)

The strategy is to remove the source code enclosed in tags.

<table>
  <thead>
    <tr>
      <th>Original</th>
      <th>Removed</th>
    </tr>
  </thead>
  <tr>
    <td>
      (delimiters: <code>&lt;!--</code>, <code>--&gt;</code>)
      <pre><code>Content A
&lt;!-- tag --&gt;
Removal content
&lt;!-- /tag --&gt;
Content B
</code></pre>
    </td>
    <td>
      <pre><code>Content A
Content B
</code></pre>
    </td>
  </tr>
</table>

### Unwrap Block

This strategy can unwrap nested source code.

For example, unwrapping an ‘If block’ ensures that the source code in the block is always executed.

To use this strategy, add a `unwrap-block` attribute to the tag.

<table>
  <thead>
    <tr>
      <th>Original</th>
      <th>Removed</th>
    </tr>
  </thead>
  <tr>
    <td>
      (delimiters: <code>/* &lt;</code>, <code>&gt; */</code>)
      <pre><code>/* &lt;tag unwrap-block&gt; */
if (foo === "bar") {
  console.log("baz");
}
/* &lt;/tag&gt; */
</code></pre>
    </td>
    <td>
      <pre><code>console.log("baz");
</code></pre>
    </td>
  </tr>
</table>

Unwrap Block makes the line immediately after the start tag and the line immediately before the end tag the extent of the block.
If the removal target does not exist immediately after the start tag and immediately before the end tag, it is not removed.

```javascript
/* This source code is removed */

/* <time-limited to="2000-01-01 00:00:00" unwrap-block> */
if (released) {
  console.log("Released!");
}
/* </time-limited> */

/* This source code is NOT removed */

/* <time-limited to="2000-01-01 00:00:00" unwrap-block> */
console.log("Released!");
/* </time-limited> */

/* <time-limited to="2000-01-01 00:00:00" unwrap-block> */ console.log("Released"); /* </time-limited> */
```

### Skip

If the `skip` attribute is given, no action is taken even if the removal condition is satisfied.

```html
<html>
  <body>
    <h1>Hello World</h1>
    <!-- Removal conditions satisfied, but not removed -->
    <!-- <time-limited to="2000-01-01 00:00:00 skip> -->
    <p>Feature 1 will be released soon.</p>
    <!-- </time-limited> -->
  </body>
</html>
```

## Supported Character Code

Only UTF-8 character encoding is supported. If you want to enter other character codes, you need to convert it.

```
iconv -f euc-jp -t utf-8 ./sample-code.php | chiritori | iconv -f utf-8 -t euc-jp
```
