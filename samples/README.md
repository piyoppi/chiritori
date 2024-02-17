# Samples

Run Chiritori with the sample code in this directory to experience how it works.

## JavaScript

```
chiritori --filename=sample-code.js  --time-limited-delimiter-start="// --" --time-limited-delimiter-end="-- //"
```

## PHP (EUC-JP encoding)

```
iconv -f euc-jp -t utf-8 sample-code.eucjp.php | chiritori --time-limited-delimiter-start="# <" --time-limited-delimiter-end="> #"
```