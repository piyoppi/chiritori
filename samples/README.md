# Samples

Run Chiritori with the sample code in this directory to experience how it works.

## HTML

```
chiritori --filename=./samples/sample-code.html
```

## JavaScript

```
chiritori --filename=./samples/sample-code.js --delimiter-start="// --" --delimiter-end="-- //" --time-limited-tag-name="time-limited-code"
```

## PHP (EUC-JP encoding)

```
iconv -f euc-jp -t utf-8 sample-code.eucjp.php | chiritori --time-limited-delimiter-start="# <" --time-limited-delimiter-end="> #"
```
