# 🧹 Chiritori

<img src="./images/chiritori.png" alt="logo" width="400" height="323">

'Chiritori' is a tool for removing time-limited source code.

It can be used to remove content such as campaigns and to remove conditional branches of feature flags.

- [Pre-Tag source code](#removal-tags-in-source-code) (e.g. using comments) and remove them when required
- [Remove code blocks](#range-default) or [open conditional branches](#unwrap-block) (e.g. if blocks).
- Available in [any programming language](#delimiter-settings)
- Support for GitHub Actions integration

> [!TIP]
> We also provide [actions-chiritori](https://github.com/piyoppi/actions-chiritori), available on GitHub Actions.
>
> Support for integration into the development workflow.

## Contents

- [Demo](#demo)
- [Install](#install)
- [Removal Tags in source code](#removal-tags-in-source-code)
- [Command Arguments](#command-arguments)
  - [Input source code](#input-source-code)
  - [Output source code](#output-source-code)
  - [Delimiter Settings](#delimiter-settings)
  - [Help](#help)
- [Removal Tags](#removal-tags)
  - [`time-limited`](#time-limited)
  - [`removal-marker`](#removal-marker)
- [Removal Strategy](#removal-strategy)
  - [Range](#range-default)
  - [Unwrap Block](#unwrap-block)
  - [Skip](#skip)
- [Supported Character Code](#supported-character-code)

## Demo

- [Demo in the browser](https://piyoppi.github.io/chiritori-web/)
- [/sample directory](./samples/) of this repository.

## Install

### Homebrew

```
$ brew install piyoppi/tap/chiritori
```

### Manually

Binaries are available on [GitHub Releases](https://github.com/piyoppi/chiritori/releases).

## Removal Tags in source code

Chiritori detects removable source code by "Removal Tag" in the source code.
Removal Tag is enclosed by start and end delimiters and consist of a tag name and attributes.

The start and end delimiters can be set to any string (See [Delimiter Settings](#delimiter-settings)).
```
           <!-- < tag-name attribute1="value"  > -->
           ^^^^^^ ^^^^^^^^ ^^^^^^^^^^^^^^^^^^  ^^^^^
  start-delimiter tag name      attribute      end-delimiter
```

The source code to be removed is enclosed in start and end Removal Tags.
The closing tag is the tag name with a slash prefixed.

```
<!-- <tag-name attribute1="value"> -->
Removal source code
<!-- </tag-name> -->
```

The `"c"` attribute is a reserved word for inline comments. You can write any comment to the value of the `"c"`  attribute.

```html
/* < time-limited to="2001-01-03 23:59:59"
 * c="New Year's greetings are displayed until 3 January."
 * > */
<h1>Heppy New Year!</h1>
/* < /time-limited > */
```

For more information on available removal tags and strategies, See [Removal Tags](#removal-tags) and [Removal Strategy](#removal-strategy)

## Command Arguments

### Input source code

Target source code can be provided to Chiritori via command line arguments or standard input.

```
$ chiritori --filename=./source-code.js

$ cat ./source-code.js | chiritori
```

### Output source code

If the `--output` option is not specified, the processed source code is output to standard output.

```
# The processed source code is written to filesystem.
$ chiritori --filename=./source-code.js --output=./processed-source-code.js

# The processed source code is output to standard output.
$ chiritori --filename=./source-code.js
```

### Delimiter Settings

The delimiter for representing an element can be changed via command line arguments.
This allows any programming language to be used.

```
$ chiritori --delimiter-start="// --" --delimiter-end="-- //" --filename=./code.js
```

### List removal targets

`--list-all` option provides a list of targets for removal.

```
$ chiritori --list-all --filename=./code.js
```

### Help

More information on Command Line Interface arguments can be found in the `chiritori --help` command.

```
$ chiritori --help
```

## Removal Tags

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

Removal targets can also be specified using a configuration file.

```
chiritori --filename=./samples/sample-code.js --removal-marker-target-config=./config.txt
```

The configuration file specifies the target for each new line.
If the contents of config.txt are as follows.

```
feature1
feature2
```

It is equivalent to the following commands.

```
chiritori --filename=./samples/sample-code.js --removal-marker-target-name="feature1" --removal-marker-target-name="feature2"
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
/* ----------- This source code is removed ----------- */

/* <time-limited to="2000-01-01 00:00:00" unwrap-block> */
if (released) {
  console.log("Released!");
}
/* </time-limited> */

/* --------- This source code is NOT removed --------- */

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
