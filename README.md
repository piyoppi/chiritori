# 🧹 chiritori

'chiritori' is a tool for finding and removing time-limited source code.

## Usage

```
chiritori --filename=./sample-code.html
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
  </body>
</html>
```

(Converted)
```html
<html>
  <body>
    <h1>Hello World</h1>
  </body>
</html>
```

The delimiter for representing an element can be changed via command line arguments.
This allows any programming language to be used.

```
chiritori \
  --filename=./sample-code.js \
  --time-limited-delimiter-start="// --" \
  --time-limited-delimiter-end="-- //"
```

Only UTF-8 character encoding is supported. If you want to enter other character codes, you need to convert it.

```
iconv -f euc-jp -t utf-8 ./sample-code.php | chiritori
```