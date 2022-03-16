#!/bin/bash
cat <<EOF | clang -xc -c -o tmp2.o -
#include <stdio.h>
#include <stdlib.h>
int add(int x, int y) { return x+y; }
void alloc4(int **p, int a, int b, int c, int d) {
    int *int_ptr = (int *)malloc(sizeof(int) * 4);
    int_ptr[0] = a;
    int_ptr[1] = b;
    int_ptr[2] = c;
    int_ptr[3] = d;
 
    *p = int_ptr;
}
EOF

assert() {
  expected="$1"
  input="$2"

  RUST_BACKTRACE=1 cargo run -- "$input" > tmp.s
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
assert 42 'int main() { return 42; }'

# 四則演算
assert 47 'int main() { return 5+6*7; }'
assert 15 'int main() { return 5*(9-6); }'
assert 4 'int main() { return (3+5)/2; }'
assert 10 'int main() { return -10+20; }'

# 比較演算
assert 1 'int main() { return 2<3; }'
assert 1 'int main() { return 2<=2; }'
assert 1 'int main() { return 3>2; }'
assert 1 'int main() { return 2>=2; }'
assert 1 'int main() { return 2==2; }'
assert 1 'int main() { return 3!=2; }'

assert 0 'int main() { return 3<2; }'
assert 0 'int main() { return 3<=2; }'
assert 0 'int main() { return 2>3; }'
assert 0 'int main() { return 2>=3; }'
assert 0 'int main() { return 2==3; }'
assert 0 'int main() { return 2!=2; }'

# 1文字変数
assert 42 'int main() { int a; a = 42; return a; }'
assert 21 'int main() { int a; int b; int c; a = 5; b = 20; c = 4; return a + b - c; }'

# 複数文字変数
assert 42 'int main() { int foo; foo = 42; return foo; }'
assert 21 'int main() { int foo; int bar; int baz; foo = 5; bar = 20; baz = 4; return foo + bar - baz; }'

# return
assert 42 'int main() { int foo; foo = 42; return foo; }'
assert 5 'int main() { int a; int b; a = 5; return a; b = 20; }'

# if else
assert 42 'int main() { int a; a = 10; if (a == 10) return 42; }'
assert 42 'int main() { int a; a = 1; if (a != 10) return 42; return 24; }'
assert 42 'int main() { int a; a = 10; if (a == 10) return 42; else return 24; }'
assert 24 'int main() { int a; a = 10; if (a != 10) return 42; else return 24; }'

# while
assert 10 'int main() { int a; a = 0; while (a != 10) a = a + 1; return a; }'
assert 1 'int main() { int a; a = 0; while (a == 0) a = a + 1; return a; }'

# for
assert 10 'int main() { int a; a = 0; for (a = 0; a < 10; a = a + 1) 42; return a; }'
assert 10 'int main() { int a; a = 0; for (; a < 10; a = a + 1) 42; return a; }'

# block
assert 30 'int main() { int a; int b; int c; a = 0; b = 0; c = 0; if (a == 0) { b = 10; c = 20; } return b + c; }'
assert 30 'int main() { int a; int b; int c; a = 0; b = 0; c = 0; if (a != 0) {} else { b = 10; c = 20; } return b + c; }'
assert 10 'int main() { int a; int b; a = 0; b = 0; for (a = 0; a < 10;) { a = a + 1; b = b + 1; } return b; }'

# funcall
assert 3 'int main() { int a; int b; int c; a = 1; b = 2; c = add(a, b); return c; }'
assert 10 'int main() { int c; c = add(add(1, 2), add(3, 4)); return c; }'

# fndef
assert 42 'int foo() { return 42; } int main() { return foo(); }' 
assert 24 'int fact(int a) { if (a == 0) { return 1; } else { return a * fact(a - 1); }  } int main() { return fact(4); }' 
assert 55 'int fib(int a) { if (a == 0) { return 0; } else if (a == 1) { return 1; }  else { return fib(a - 1) + fib(a - 2); } } int main() { return fib(10); }' 
assert 42 'int foo(int a) { int b; b = a + 1; return b; } int main() { return foo(41); }' 

# addr
assert 3 'int main() { int x; int* y; x = 3; y = &x; return *y; }' 

# int ptr
assert 3 'int main() { int x; int *y; y = &x; *y = 3; return x; }' 

# pointer arithmetic
assert 4 'int main() { int *p; alloc4(&p, 1, 2, 4, 8); int *q; q = p + 2; return *q; }'
assert 8 'int main() { int *p; alloc4(&p, 1, 2, 4, 8); int *q; q = p + 3; return *q; }'
assert 2 'int main() { int *p; alloc4(&p, 1, 2, 4, 8); int *q; q = p + 2; q = q - 1; return *q; }'

# sizeof
assert 4 'int main() { int x; x = sizeof(x); return x; }'
assert 4 'int main() { int x; x = sizeof x; return x; }'
assert 8 'int main() { int x; int *y; x = sizeof(y); return x; }'
assert 8 'int main() { int x; int *y; x = sizeof y ; return x; }'

# array var
assert 3 'int main() { int x[3][3]; int y; y = 3; return y; }'
assert 3 'int main() { int a[2]; *a = 1; *(a + 1) = 2; int *p; p = a; return *p + *(p + 1); }'
assert 3 'int main() { int x[3]; *x=3; *(x+1)=4; *(x+2)=5; return *x; }'
assert 4 'int main() { int x[3]; *x=3; *(x+1)=4; *(x+2)=5; return *(x+1); }'
assert 5 'int main() { int x[3]; *x=3; *(x+1)=4; *(x+2)=5; return *(x+2); }'

# multi dimension array
assert 0 'int main() { int x[2][3]; int *y; y=&x; *y=0; return **x; }'
assert 1 'int main() { int x[2][3]; int *y; y=&x; *(y+1)=1; return *(*x+1); }'
assert 2 'int main() { int x[2][3]; int *y; y=&x; *(y+2)=2; return *(*x+2); }'
assert 3 'int main() { int x[2][3]; int *y; y=&x; *(y+3)=3; return **(x+1); }'
assert 4 'int main() { int x[2][3]; int *y; y=&x; *(y+4)=4; return *(*(x+1)+1); }'
assert 5 'int main() { int x[2][3]; int *y; y=&x; *(y+5)=5; return *(*(x+1)+2); }'

# array subscript
assert 1 'int main() { int x[3]; x[0] = 1; x[1] = 2; x[2] = 4; return x[0]; }'
assert 2 'int main() { int x[3]; x[0] = 1; x[1] = 2; x[2] = 4; return x[1]; }'
assert 4 'int main() { int x[3]; x[0] = 1; x[1] = 2; x[2] = 4; return x[2]; }'
assert 5 'int main() { int x[2][3]; x[1][2] = 5; return x[1][2]; }'

# global var
assert 42 'int foo; int main() { foo = 42; return foo; }' 
assert 42 'int *y; int main() { int x; y = &x; *y = 42; return x; }' 
assert 42 'int x[2][3]; int main() { x[1][2] = 42; return x[1][2]; }' 
assert 42 'int x[2][3]; int main() { x[0][0] = 42; return **x; }' 

# char
assert 3 'int main() { char x[3]; x[0] = -1; x[1] = -2; int y; y = 4; return x[0] + y; }' 
 
echo OK