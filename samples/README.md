# Samples

Run Chiritori with the sample code in this directory to experience how it works.

## HTML

```
chiritori --filename=./sample-code.html
```

## JavaScript

```
chiritori --filename=./sample-code.js --delimiter-start="// --" --delimiter-end="-- //" --time-limited-tag-name="time-limited-code"
```

## PHP (EUC-JP encoding)

```
iconv -f euc-jp -t utf-8 sample-code.eucjp.php | chiritori --delimiter-start="# <" --delimiter-end="> #"
```
