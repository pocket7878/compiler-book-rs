#!/bin/bash
assert() {
  expected="$1"
  input="$2"

  cargo run -- "$input" > tmp.s
  cc -o tmp tmp.s
  ./tmp
  actual="$?"

  if [ "$actual" = "$expected" ]; then
    echo "$input => $actual"
  else
    echo "$input => $expected expected, but got $actual"
    exit 1
  fi
}

assert 47 '5+6*7'
assert 15 '5*(9-6)'
assert 4 '(3+5)/2'
assert 10 '-10+20'

assert 1 '2<3'
assert 1 '2<=2'
assert 1 '3>2'
assert 1 '2>=2'

assert 0 '3<2'
assert 0 '3<=2'
assert 0 '2>3'
assert 0 '2>=3'

echo OK