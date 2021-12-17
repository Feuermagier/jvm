public class Test {
    static int a;
	public static void main() {
        foo(5);
        foo(-3);
    }
    
    static void foo(int x) {
        a = x;
    }
}