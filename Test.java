public class Test {
    static int a;
	public static void main() {
        foo(5);
        foo(10000);
    }
    
    static void foo(int x) {
        a = x;
    }
}