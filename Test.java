public class Test {
    static long a;
	public static void main() {
		long x = -Long.MAX_VALUE;
        foo(5);
        foo(x);
    }
    
    static void foo(long x) {
        a = x;
    }
}