public class Test {
    static double a;
	public static void main() {
		Test x = new Test2();
        x.foo(3.14);
    }
    
    public void foo(double x) {
        a = x;
    }
}