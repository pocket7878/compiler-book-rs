#!/bin/bash
cat <<EOF | clang -xc -c -o tmp2.o -
#include <stdio.h>
int add(int x, int y) { return x+y; }
void foo(x, y) { printf("x = %d, y = %d\n", x, y); }
EOF

assert() {
  expected="$1"
  input="$2"

  cargo run -- "$input" > tmp.s
  cc -c tmp.s
  cc -o tmp tmp.o tmp2.o
  ./tmp
  actual="$?"

  if [ "$actual" = "$expected" ]; then
    echo "$input => $actual"
  else
    echo "$input => $expected expected, but got $actual"
    exit 1
  fi
}

# 値を返すだけ
assert 42 'main() { return 42; }'

# 四則演算
assert 47 'main() { return 5+6*7; }'
assert 15 'main() { return 5*(9-6); }'
assert 4 'main() { return (3+5)/2; }'
assert 10 'main() { return -10+20; }'

# 比較演算
assert 1 'main() { return 2<3; }'
assert 1 'main() { return 2<=2; }'
assert 1 'main() { return 3>2; }'
assert 1 'main() { return 2>=2; }'
assert 1 'main() { return 2==2; }'
assert 1 'main() { return 3!=2; }'

assert 0 'main() { return 3<2; }'
assert 0 'main() { return 3<=2; }'
assert 0 'main() { return 2>3; }'
assert 0 'main() { return 2>=3; }'
assert 0 'main() { return 2==3; }'
assert 0 'main() { return 2!=2; }'

# 1文字変数
assert 42 'main() { a = 42; return a; }'
assert 21 'main() { a = 5; b = 20; c = 4; return a + b - c; }'

# 複数文字変数
assert 42 'main() { foo = 42; return foo; }'
assert 21 'main() { foo = 5; bar = 20; baz = 4; return foo + bar - baz; }'

# return
assert 42 'main() { foo = 42; return foo; }'
assert 5 'main() { a = 5; return a; b = 20; }'

# if else
assert 42 'main() { a = 10; if (a == 10) return 42; }'
assert 42 'main() { a = 1; if (a != 10) return 42; return 24; }'
assert 42 'main() { a = 10; if (a == 10) return 42; else return 24; }'
assert 24 'main() { a = 10; if (a != 10) return 42; else return 24; }'

# while
assert 10 'main() { a = 0; while (a != 10) a = a + 1; return a; }'
assert 1 'main() { a = 0; while (a == 0) a = a + 1; return a; }'

# for
assert 10 'main() { a = 0; for (a = 0; a < 10; a = a + 1) 42; return a; }'
assert 10 'main() { a = 0; for (; a < 10; a = a + 1) 42; return a; }'

# block
assert 30 'main() { a = 0; b = 0; c = 0; if (a == 0) { b = 10; c = 20; } return b + c; }'
assert 30 'main() { a = 0; b = 0; c = 0; if (a != 0) {} else { b = 10; c = 20; } return b + c; }'
assert 10 'main() { a = 0; b = 0; for (a = 0; a < 10;) { a = a + 1; b = b + 1; } return b; }'

# funcall
assert 3 'main() { a = 1; b = 2; c = add(a, b); return c; }'
assert 10 'main() { c = add(add(1, 2), add(3, 4)); return c; }'

echo OK