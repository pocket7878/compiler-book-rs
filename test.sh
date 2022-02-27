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

# 四則演算
assert 47 '5+6*7;'
assert 15 '5*(9-6);'
assert 4 '(3+5)/2;'
assert 10 '-10+20;'

# 比較演算
assert 1 '2<3;'
assert 1 '2<=2;'
assert 1 '3>2;'
assert 1 '2>=2;'
assert 1 '2==2;'
assert 1 '3!=2;'

assert 0 '3<2;'
assert 0 '3<=2;'
assert 0 '2>3;'
assert 0 '2>=3;'
assert 0 '2==3;'
assert 0 '2!=2;'

# 1文字変数
assert 42 'a = 42; a;'
assert 21 'a = 5; b = 20; c = 4; a + b - c;'

# 複数文字変数
assert 42 'foo = 42; foo;'
assert 21 'foo = 5; bar = 20; baz = 4; foo + bar - baz;'

# return
assert 42 'foo = 42; return foo;'
assert 5 'a = 5; return a; b = 20;'

echo OK