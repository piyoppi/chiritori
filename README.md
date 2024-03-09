# 🧹 chiritori

'chiritori' is a tool for finding and removing time-limited source code.

## Usage

```
chiritori --filename=./samples/sample-code.html
```

This command will remove the parts of the following source code bounded by the "time-limited" element.

(Original)
```html
<html>
  <body>
    <h1>Hello World</h1>
    <!-- time-limited to="2024/02/15 12:00:00" -->
    <h2>Campaign until 2024/02/15 12:00</h2>
    <!-- /time-limited -->
    <!-- time-limited to="9999/12/31 23:59:59" -->
    <h2>Campaign until 9999/12/31</h2>
    <!-- /time-limited -->
  </body>
</html>
```

(Converted)
```html
<html>
  <body>
    <h1>Hello World</h1>
    <!-- time-limited to="9999/12/31 23:59:59" -->
    <h2>Campaign until 9999/12/31</h2>
    <!-- /time-limited -->
  </body>
</html>
```

For more options, run the following commands.

```
chiritori --help
```

The delimiter for representing an element can be changed via command line arguments.
This allows any programming language to be used.

```
chiritori \
  --filename=./samples/sample-code.js \
  --delimiter-start="// --" \
  --delimiter-end="-- //" \
  --time-limited-tag-name="time-limited-code"
```

### Supported character code

Only UTF-8 character encoding is supported. If you want to enter other character codes, you need to convert it.

```
iconv -f euc-jp -t utf-8 ./sample-code.php | chiritori | iconv -f utf-8 -t euc-jp
```
