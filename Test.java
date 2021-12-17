public class Test {
    static long a;
	public static void main() {
        foo(5);
        foo(Long.MAX_VALUE);
    }
    
    static void foo(long x) {
        a = x;
    }
}