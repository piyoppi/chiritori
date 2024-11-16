# Samples

Run Chiritori with the sample code in this directory to experience how it works.

## Remove source codes

### HTML

```
chiritori --filename=./sample-code.html
```

### JavaScript

```
chiritori --filename=./samples/sample-code.js --delimiter-start="// --" --delimiter-end="-- //" --time-limited-tag-name="time-limited-code" --removal-marker-target-name="awesome-feature"
```

### PHP (EUC-JP encoding)

```
iconv -f euc-jp -t utf-8 sample-code.eucjp.php | chiritori --delimiter-start="# <" --delimiter-end="> #"
```

## List source code to be removed

To list the codes to be removed, run the following command.

```
$ chiritori --filename=./sample-code.html -l
```

To list pending source code, run the following command.

```
$ chiritori --filename=./sample-code.html --list-all
```
