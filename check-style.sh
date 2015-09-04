#!/bin/bash

# config
COLS=100
FOLDER="src"
FILES="*.rs"


# Exit script on the first error
set -o errexit -o nounset

ERROR=0
### Trailing Whitespaces ===========================
echo ""
echo "=== Searching for lines with trailing whitespace... ==================="
if egrep --include $FILES -rHn " +$" $FOLDER ; then
    echo ""
    echo "!!! Some lines were found. Please remove the trailing whitespace!"
    ERROR=1
else
    echo "=== None found! :-)"
fi


### Trailing newlines ===============================
echo ""
echo "=== Searching for files without trailing newline... ==================="
FOUND=0
for f in $(find $FOLDER -name $FILES); do
    lastline=$(tail -n 1 $f; echo x)
    lastline=${lastline%x}
    if [ "${lastline: -1}" != $'\n' ] ; then
        echo "! Has no single trailing newline: $f"
        FOUND=1
    fi
done

if [ $FOUND -eq 0 ] ; then
    echo "=== None found! :-)"
else
    echo ""
    echo "!!! Some files were found. Please add a single trailing newline!"
    ERROR=1
fi


### char limit ===================================
echo ""
echo "=== Searching for files with too long lines... ========================"
FOUND=0
for f in $(find $FOLDER -name $FILES); do
    if [ $(wc -L $f | cut -d" " -f1) -gt $COLS ] ; then
        echo "! Line with more than $COLS chars in $f"
        FOUND=1
    fi
done

if [ $FOUND -eq 0 ] ; then
    echo "=== None found! :-)"
else
    echo ""
    echo "!!! Some files were found. Please shorten those lines!"
    ERROR=1
fi

test $ERROR == 0
