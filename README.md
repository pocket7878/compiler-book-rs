# compiler-book-rs

[『低レイヤを知りたい人のためのCコンパイラ作成入門』](https://www.sigbus.info/compilerbook) をM1 Mac上でRustで追ってみる

# Sample

```c: sample.c
int a;
int *b;
int c[10][10];

int main()
{
	int d;
	a = 10;
	b = &d;
	*b = 10;
	int e[2][3];
	e[1][2] = 2;
	c[9][9] = 10;
	/*
	int c;
	b = &c;
	*b = 20;
	*/
	// Line comment
	/*
	 * Block comment
	 * Code can be insize
	 * printf("hello, world\n");
	 */
	/* Inline block comment */printf("hello, world\n"); //Inline line comment
	return a + *b + c[9][9] + d + e[1][2];
}
```

```shell
$ cargo run -- sample.c > sample.s
$ cc -o sample sample.s
$ ./sample
hello, world
$ echo $?
42
```
